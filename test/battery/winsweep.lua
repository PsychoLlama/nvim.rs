-- winsweep -- the EIGHTEENTH baselined differential.  Driven by
-- winsweep.sh, which builds the sandbox, pins $HOME /
-- $TMPDIR / $PATH and does the scrubs only the shell can see.  Read
-- that header first.
--
-- The subsystem is window.rs (8,636 lines, 8,304 unchecked) and
-- winfloat.rs (702 / 634): the whole window-manipulation family --
-- `do_window`'s letter dispatch, `win_split_ins`, `win_equal_rec`,
-- `frame_new_height`/`frame_setheight`/`frame_setwidth`,
-- `winframe_remove`/`winframe_restore`, `win_close`/`win_close_othertab`,
-- `win_enter_ext`/`win_goto`, the tabpage family, `win_setheight_win` /
-- `win_setwidth_win`, `last_status`/`command_height`, and every float
-- path (`win_new_float`, `win_config_float`, `ui_ext_win_position`,
-- `win_float_remove`, `win_check_anchored_floats`).
--
-- The gap this closes: B20's survey S5, holes 1 and 2.  Of seventeen
-- baselined differentials, every `wincmd` letter appearing anywhere was
-- navigation or sizing (`j w l p t k | _ o c b zz`) -- ZERO of
-- `x r R H J K L T = + - < > s v n q ^`; nothing anywhere read
-- `winrestcmd()`; and every `relative=` in every oracle was `'editor'`
-- with `anchor` appearing exactly zero times.
--
-- THE DESIGN, in one paragraph.  Everything this family does that can
-- be observed is a NUMBER or a STRING: `winlayout()`'s tree,
-- `winrestcmd()`'s command list, `getwininfo()`'s geometry and
-- `nvim_win_get_config()`'s dict.  So the sweep is textual and runs in
-- ONE process, exactly as stlsweep and menusweep do.  It is NOT an
-- `-l` process, and that is the one design decision worth stating:
-- `-l` sets `silent_mode`, `full_screen` is `!silent_mode`, and
-- `did_set_cmdheight` is guarded by `full_screen` -- so under `-l`
-- `'cmdheight'` HAS NO LAYOUT EFFECT AT ALL and a whole arm of s3
-- would have measured nothing.  `--headless -c 'lua dofile(...)'`
-- keeps the process just as textual and just as loop-less while giving
-- a real `full_screen`.  Only two things need a main input loop:
-- `WinResized`/`WinScrolled`, which fire from `normal_check`'s
-- `may_trigger_win_scrolled_resized` and nowhere else (s7), and the
-- inputs that may kill the editor (s91).  Both use b19-3's `Child`.
--
-- Sections:
--   s0  defaults   the startup layout and the family's option defaults
--   s1  wincmd     every letter of `do_window`, with counts
--   s2  split      split/close/only/hide/new/quit x the option matrix
--   s3  resize     `:resize`, `:vertical resize`, the geometry options
--                  and the screen-shape options (cmdheight/laststatus/
--                  showtabline/lines/columns)
--   s4  tabpage    tabnew/tabclose/tabonly/tabmove/tabnext, `wincmd T`,
--                  and win_goto across tabs
--   s5  float      relative x anchor x zindex x border x title/footer
--                  x bufpos -- the matrix that was in NO oracle
--   s6  config     `nvim_win_set_config` moves, reanchors, reparents,
--                  float<->split conversion, hide/focusable/fixed
--   s7  auorder    Win{NewPre,New,Closed,Enter,Leave,Resized,Scrolled}
--                  and Tab{New,Enter,Leave,Closed} ORDER, in a child
--   s91 crashprobe the inputs that may kill the editor, one child each
--
-- Every section ends with a `## <name> rows=N` line.  A section that
-- goes silently empty otherwise looks exactly like a healthy one.
--
-- DETERMINISM.  Window, buffer and tabpage handles are process-global
-- counters, so a case that opens one window renumbers every later case.
-- NOTHING RAW IS EVER PRINTED: `snap()` renumbers every handle to an
-- ordinal (`w1`, `b2`, `t3`) assigned in `getwininfo()` order, which is
-- tab order then winnr order -- i.e. the order that IS behaviour.  That
-- makes an inserted case a pure addition instead of a re-baseline of
-- everything below it (b19-14's hazard, paid off rather than avoided).

local work = assert(os.getenv('WIN_WORK'), 'WIN_WORK unset')
local runtime = os.getenv('VIMRUNTIME') or ''
local script = debug.getinfo(1, 'S').source:sub(2)

local only = os.getenv('WINSWEEP_ONLY')
if only == '' then
  only = nil
end
local trace = os.getenv('WINSWEEP_TRACE') == '1'

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
  limit = limit or 900
  if #text <= limit then
    return text
  end
  return text:sub(1, limit) .. string.format('...<+%d>', #text - limit)
end

--- Escape to one printable line.  Border characters are box-drawing
--- multibyte and `shadow`'s cells are spaces with a highlight name; a
--- raw byte in the report would make `diff` call it binary.
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
  assert(io.open(assert(os.getenv('WIN_STRUCT'), 'WIN_STRUCT unset'), 'w'))

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
    local v = value[k]
    if v == nil then
      v = value[tonumber(k)]
    end
    parts[#parts + 1] = q(k) .. ':' .. canon(v)
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
  s = s:gsub('^[^\n]-winsweep%.lua:%d+: ', '')
  s = s:gsub('^%[string "[^"]*"%]:%d+: ', '')
  s = s:gsub('^nvim_exec2%(%), line %d+: ', '')
  s = s:gsub('\r?\n', ' | ')
  return cap(scrub(s))
end

--- Serialise a small table back into Lua source, so the parent can hand
--- a child's helper its arguments without a second roundtrip.
local tolua
function tolua(value)
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

--- The option baseline.  Every case starts from exactly this.
---
--- `'equalalways'` defaults ON and `'winwidth'` to 20, both of which
--- silently rewrite a layout; they are pinned, not inherited, so an
--- upstream default change shows up in s0 as one row rather than in
--- every row of s2.
local BASEOPT = {
  'set noswapfile nobackup nowritebackup hidden',
  'set report=9999 shortmess=aoOtTIcCF nomore belloff=all',
  'set lines=24 columns=80 cmdheight=1 laststatus=2 showtabline=1',
  'set noequalalways eadirection=both nosplitbelow nosplitright',
  'set winheight=1 winwidth=1 winminheight=1 winminwidth=1',
  'set noruler noshowcmd nonumber norelativenumber nowrap',
  'set scrolloff=0 sidescrolloff=0 nostartofline',
  'set eventignore= verbose=0 nomodeline modelines=0',
  'set mouse= mousemodel=popup_setpos guicursor=',
  'set switchbuf= previewheight=12 helpheight=20 winbar=',
  'set fillchars= listchars= statusline= tabline= rulerformat=',
}

local function baseopts()
  for _, line in ipairs(BASEOPT) do
    pcall(vim.api.nvim_exec2, line, { output = false })
  end
  -- Window-local geometry pins have to be cleared per window, not once.
  for _, w in ipairs(vim.api.nvim_list_wins()) do
    pcall(vim.api.nvim_set_option_value, 'winfixheight', false, { win = w })
    pcall(vim.api.nvim_set_option_value, 'winfixwidth', false, { win = w })
    pcall(vim.api.nvim_set_option_value, 'winfixbuf', false, { win = w })
    pcall(vim.api.nvim_set_option_value, 'previewwindow', false, { win = w })
  end
end

--- Eight scratch buffers, built ONCE.  A case that made its own would
--- push the buffer counter by four per case and leak ~4,000 buffers
--- over the sweep; reusing a pool keeps memory flat and keeps the
--- ordinal map stable.  Sixty lines each, so `w_topline` has somewhere
--- to go when `frame_new_height` shrinks a window.
local POOL = {}
local function makepool()
  for i = 1, 8 do
    local buf = vim.api.nvim_create_buf(false, true)
    local lines = {}
    for n = 1, 60 do
      lines[n] = string.format('buf%d line %02d', i, n)
    end
    vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
    POOL[i] = buf
  end
end

-- ------------------------------------------------------------- answers

--- One canonical picture of the whole editor's window state.
---
--- Handles are renumbered here and NOWHERE else.  `getwininfo()` walks
--- tabpages in order and windows in `winnr` order inside each, which is
--- precisely the order the user sees, so the ordinals themselves are an
--- assertion about window ORDER as well as a de-noising.
--- TWO redraws, before every answer.
---
--- A float's SCREEN position is `w_winrow`/`w_wincol`, written by
--- `ui_ext_win_position` from `win_ui_flush(true)` -- which is called
--- from `update_screen()` and from nowhere else.  Without it every
--- `anchor`, every `fixed` and every clamp in that function answers the
--- config's own `row`/`col` back, so the whole relative x anchor matrix
--- would have been 96 identical rows.  It takes TWO: the first redraw
--- allocates the float's grid (`win_grid_alloc` runs while drawing, and
--- `win_ui_flush` skips a window whose `w_grid_alloc.chars` is null),
--- the second positions it.  This is also why the sweep runs from
--- `VimEnter` and not from `-c`: `exe_commands` runs while
--- `RedrawingDisabled` is still 1, so under `-c` a `:redraw` does
--- nothing at all.
local function paint()
  pcall(vim.api.nvim_exec2, 'redraw', { output = false })
  pcall(vim.api.nvim_exec2, 'redraw', { output = false })
end

local function snap()
  paint()
  local infos = vim.fn.getwininfo()
  local wmap, bmap, tmap = {}, {}, {}
  local nb = 0
  for i, t in ipairs(vim.api.nvim_list_tabpages()) do
    tmap[t] = 't' .. i
  end
  for i, wi in ipairs(infos) do
    wmap[wi.winid] = 'w' .. i
    if not bmap[wi.bufnr] then
      nb = nb + 1
      bmap[wi.bufnr] = 'b' .. nb
    end
  end
  local function W(id)
    return wmap[id] or ('?' .. (id == 0 and 0 or 1))
  end

  local lay = {}
  local function relabel(node)
    if type(node) ~= 'table' then
      return node
    end
    if node[1] == 'leaf' then
      return { 'leaf', W(node[2]) }
    end
    local kids = {}
    for i, k in ipairs(node[2]) do
      kids[i] = relabel(k)
    end
    return { node[1], kids }
  end
  local ntabs = vim.fn.tabpagenr('$')
  for t = 1, ntabs do
    lay[t] = relabel(vim.fn.winlayout(t))
  end

  local ws, cfgs = {}, {}
  for i, wi in ipairs(infos) do
    local id = wi.winid
    local ok, cur = pcall(vim.api.nvim_win_get_cursor, id)
    local sp = {}
    if wi.tabnr == vim.fn.tabpagenr() then
      local okp, r = pcall(vim.fn.screenpos, id, ok and cur[1] or 1, 1)
      if okp and type(r) == 'table' then
        sp = { r.row or 0, r.col or 0 }
      end
    end
    local function wo(name)
      local o, v = pcall(vim.api.nvim_get_option_value, name, { win = id })
      return o and (v and 1 or 0) or -1
    end
    ws[i] = {
      id = 'w' .. i,
      nr = wi.winnr,
      tab = wi.tabnr,
      buf = bmap[wi.bufnr],
      h = wi.height,
      w = wi.width,
      row = wi.winrow,
      col = wi.wincol,
      st = wi.status_height,
      bar = wi.winbar,
      toff = wi.textoff,
      tl = wi.topline,
      bl = wi.botline,
      lc = wi.leftcol,
      cl = ok and cur[1] or -1,
      cc = ok and cur[2] or -1,
      sp = sp,
      wfh = wo('winfixheight'),
      wfw = wo('winfixwidth'),
    }
    local okc, c = pcall(vim.api.nvim_win_get_config, id)
    if okc then
      if c.win ~= nil then
        c.win = W(c.win)
      end
      cfgs[i] = c
    else
      cfgs[i] = { ERR = errtext(c) }
    end
  end

  return {
    cur = W(vim.api.nvim_get_current_win()),
    nw = vim.fn.winnr('$'),
    alt = vim.fn.winnr('#'),
    tab = vim.fn.tabpagenr(),
    ntab = ntabs,
    atab = vim.fn.tabpagenr('#'),
    lay = lay,
    rc = vim.fn.winrestcmd(),
    lines = vim.o.lines,
    cols = vim.o.columns,
    ch = vim.o.cmdheight,
    ls = vim.o.laststatus,
    stal = vim.o.showtabline,
    ws = ws,
    cfg = cfgs,
  }
end

local function laystr(node)
  if type(node) ~= 'table' then
    return tostring(node)
  end
  if node[1] == 'leaf' then
    return node[2]
  end
  local parts = {}
  for i, k in ipairs(node[2]) do
    parts[i] = laystr(k)
  end
  return (node[1] == 'col' and 'C[' or 'R[') .. table.concat(parts, ' ') .. ']'
end

--- The compact rendering.  The struct row carries EVERY field of every
--- window and the full `nvim_win_get_config` dict; this is the half a
--- human reads in a diff, so it keeps the geometry and drops what the
--- config repeats.
local function snapstr(s)
  local out = {
    string.format(
      'cur=%s n=%d alt=%d tab=%d/%d atab=%d scr=%dx%d ch=%d ls=%d stal=%d',
      s.cur,
      s.nw,
      s.alt,
      s.tab,
      s.ntab,
      s.atab,
      s.lines,
      s.cols,
      s.ch,
      s.ls,
      s.stal
    ),
  }
  for i, l in ipairs(s.lay) do
    out[#out + 1] = 'L' .. i .. '=' .. laystr(l)
  end
  out[#out + 1] = 'RC=' .. s.rc
  for i, w in ipairs(s.ws) do
    out[#out + 1] = string.format(
      '%s{n%d t%d %s %dx%d @%d,%d st%d bar%d off%d tl%d bl%d lc%d cur%d,%d sp%d,%d fx%d%d}',
      w.id,
      w.nr,
      w.tab,
      w.buf,
      w.h,
      w.w,
      w.row,
      w.col,
      w.st,
      w.bar,
      w.toff,
      w.tl,
      w.bl,
      w.lc,
      w.cl,
      w.cc,
      w.sp[1] or 0,
      w.sp[2] or 0,
      w.wfh,
      w.wfw
    )
    local c = s.cfg[i] or {}
    if c.relative == '' or c.relative == nil then
      out[#out + 1] = string.format('%s<sp=%s>', w.id, tostring(c.split))
    else
      local bits = {
        'rel=' .. tostring(c.relative),
        'anc=' .. tostring(c.anchor),
        'r=' .. tostring(c.row),
        'c=' .. tostring(c.col),
        'z=' .. tostring(c.zindex),
        'foc=' .. tostring(c.focusable),
        'hid=' .. tostring(c.hide),
        'mou=' .. tostring(c.mouse),
        'sty=' .. tostring(c.style),
      }
      if c.win then
        bits[#bits + 1] = 'win=' .. tostring(c.win)
      end
      if c.bufpos then
        bits[#bits + 1] =
          string.format('bp=%s,%s', tostring(c.bufpos[1]), tostring(c.bufpos[2]))
      end
      if c.border then
        bits[#bits + 1] = 'bd=' .. canon(c.border)
      end
      if c.title then
        bits[#bits + 1] = 'ti=' .. canon(c.title) .. '@' .. tostring(c.title_pos)
      end
      if c.footer then
        bits[#bits + 1] = 'fo='
          .. canon(c.footer)
          .. '@'
          .. tostring(c.footer_pos)
      end
      out[#out + 1] = string.format('%s<%s>', w.id, table.concat(bits, ' '))
    end
  end
  return table.concat(out, ' ')
end

--- One case: run `cmds`, then answer with the whole picture.
---
--- The command's own output is part of the answer -- `:only` on one
--- window says "Already only one window" and `wincmd P` says E441, and
--- which of the two a stage produces is exactly the behaviour a
--- mutation moves.
local function ask(label, note, value)
  label_once(label)
  local text = value.ws and snapstr(value)
    or vim.inspect(value, { newline = ' ', indent = '' })
  emit(label, '=', esc(scrub(note or '')), esc(scrub(cap(text, 4000))))
  struct(label, value)
  return value
end

local function run(label, cmds, opts)
  opts = opts or {}
  local outs = {}
  for _, c in ipairs(type(cmds) == 'table' and cmds or { cmds }) do
    local ok, res = pcall(vim.api.nvim_exec2, c, { output = true })
    if ok then
      local o = (res.output or ''):gsub('\r?\n', ' | ')
      if o ~= '' then
        outs[#outs + 1] = c .. ' -> ' .. o
      end
    else
      outs[#outs + 1] = c .. ' !! ' .. errtext(res)
    end
  end
  local note = table.concat(outs, ' ;; ')
  local s = snap()
  s.note = note
  s.cmds = type(cmds) == 'table' and cmds or { cmds }
  label_once(label)
  emit(label, '=', esc(scrub(cap(note, 400))), esc(scrub(snapstr(s))))
  struct(label, s)
  return s
end

-- -------------------------------------------------------------- layout

local LAYOUTS = {
  one = {},
  h2 = { 'split' },
  v2 = { 'vsplit' },
  col3 = { 'split', 'split' },
  row3 = { 'vsplit', 'vsplit' },
  grid4 = { 'split', 'vsplit', 'wincmd j', 'vsplit' },
  nest = { 'vsplit', 'wincmd l', 'split', 'split', 'wincmd h', 'split' },
  deep = { 'split', 'vsplit', 'split', 'vsplit', 'split' },
}
local LAYNAMES =
  { 'one', 'h2', 'v2', 'col3', 'row3', 'grid4', 'nest', 'deep' }

local function killfloats()
  for _, t in ipairs(vim.api.nvim_list_tabpages()) do
    for _, w in ipairs(vim.api.nvim_tabpage_list_wins(t)) do
      local ok, c = pcall(vim.api.nvim_win_get_config, w)
      if ok and c.relative ~= '' then
        pcall(vim.api.nvim_win_close, w, true)
      end
    end
  end
end

--- Back to one tab, one window, the option baseline, then build `lay`.
---
--- `:only` does NOT close a floating window (b19-2) -- the floats have
--- to go first and by handle, or every s5 case inherits the previous
--- one's floats.
local function reset(lay, pre)
  killfloats()
  pcall(vim.api.nvim_exec2, 'silent! tabonly!', { output = false })
  pcall(vim.api.nvim_exec2, 'silent! only!', { output = false })
  baseopts()
  vim.api.nvim_win_set_buf(0, POOL[1])
  for _, c in ipairs(pre or {}) do
    pcall(vim.api.nvim_exec2, c, { output = false })
  end
  for _, c in ipairs(LAYOUTS[lay] or {}) do
    pcall(vim.api.nvim_exec2, 'silent! ' .. c, { output = false })
  end
  -- One buffer per window, cursor mid-file: `scroll_to_fraction` and
  -- `win_fix_scroll` only have an observable effect on a window whose
  -- view is not pinned at line 1.
  local wins = vim.api.nvim_tabpage_list_wins(0)
  for i, w in ipairs(wins) do
    pcall(vim.api.nvim_win_set_buf, w, POOL[((i - 1) % 8) + 1])
    pcall(vim.api.nvim_win_set_cursor, w, { 30, 0 })
  end
  pcall(vim.api.nvim_exec2, 'silent! wincmd t', { output = false })
end

--- Focus the `n`th window of the current tab (1-based), if it exists.
local function focus(n)
  local wins = vim.api.nvim_tabpage_list_wins(0)
  if wins[n] then
    pcall(vim.api.nvim_set_current_win, wins[n])
  end
end

-- ================================================================ child
-- s7 and s91 only.  `may_trigger_win_scrolled_resized` is called from
-- `normal_check` (and from `edit`'s and the terminal's redraw), never
-- from a script, so `WinResized`/`WinScrolled` cannot fire in a process
-- that has no main loop -- however many `:resize`s it runs.
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

--- One full turn of the child's MAIN LOOP, proved by a numbered marker
--- that went through the typeahead (b19-3).  `nvim_input` is FAST and
--- `nvim_exec_lua` DEFERRED, so only a key round-trip proves the loop
--- actually turned -- and turning the loop is the whole point here,
--- because that is where `may_trigger_win_scrolled_resized` lives.
function Child:turn()
  self.tick = (self.tick or 0) + 1
  self:key('<Cmd>let g:wntick=' .. self.tick .. '<CR>')
  for _ = 1, 400 do
    if self.dead then
      return false
    end
    if self:lua('return vim.g.wntick') == self.tick then
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
      -- The crate-relative path of the panicking source file: a carve or a
      -- rewrite moves an item between files without changing what the editor
      -- does, so the location is about the port's layout, not its behaviour.
      line = line:gsub('crates/[%w_/%-%.]+%.rs', '<SRC>')
      out[#out + 1] = scrub(line)
    end
  end
  return out
end

local CPRELUDE = [[
_G.OPTS = ]] .. tolua(BASEOPT) .. [[

function _G.BASE()
  for _, c in ipairs(_G.OPTS) do pcall(vim.api.nvim_exec2, c, {output=false}) end
end
function _G.KILLFLOATS()
  for _, t in ipairs(vim.api.nvim_list_tabpages()) do
    for _, w in ipairs(vim.api.nvim_tabpage_list_wins(t)) do
      local ok, c = pcall(vim.api.nvim_win_get_config, w)
      if ok and c.relative ~= '' then pcall(vim.api.nvim_win_close, w, true) end
    end
  end
end
function _G.RESET(pre, lay)
  _G.KILLFLOATS()
  pcall(vim.api.nvim_exec2, 'silent! tabonly!', {output=false})
  pcall(vim.api.nvim_exec2, 'silent! only!', {output=false})
  _G.BASE()
  for _, c in ipairs(pre or {}) do pcall(vim.api.nvim_exec2, c, {output=false}) end
  for _, c in ipairs(lay or {}) do pcall(vim.api.nvim_exec2, 'silent! '..c, {output=false}) end
  local lines = {}
  for n = 1, 60 do lines[n] = string.format('child line %02d', n) end
  for _, w in ipairs(vim.api.nvim_tabpage_list_wins(0)) do
    local b = vim.api.nvim_create_buf(false, true)
    vim.api.nvim_buf_set_lines(b, 0, -1, false, lines)
    pcall(vim.api.nvim_win_set_buf, w, b)
  end
  _G.REMAP()
  vim.g.wnlog = {}
end

--- Window ids are a process-global counter, so a raw id in the log
--- would renumber every gesture below the one you insert.  Each
--- gesture re-seeds the map from the windows that exist when it starts
--- (`w1`..`wN`, `getwininfo()` order) and hands anything created
--- during the gesture an `n1`, `n2`, ... in first-appearance order --
--- which is creation order, i.e. behaviour.
function _G.REMAP()
  _G.WMAP = {}
  local i = 0
  for _, wi in ipairs(vim.fn.getwininfo()) do
    i = i + 1
    _G.WMAP[wi.winid] = 'w' .. i
  end
  _G.WNEW = 0
end
function _G.WID(id)
  local n = tonumber(id)
  if not n then return tostring(id) end
  if not _G.WMAP[n] then
    _G.WNEW = _G.WNEW + 1
    _G.WMAP[n] = 'n' .. _G.WNEW
  end
  return _G.WMAP[n]
end
]]

-- The pool is built BEFORE any section, not inside s0: WINSWEEP_ONLY
-- must be able to run one section on its own, and every section resets
-- through the pool.
makepool()

-- =========================================================== s0 defaults

section('s0-defaults', function()
  local names = {
    'equalalways',
    'eadirection',
    'splitbelow',
    'splitright',
    'winheight',
    'winwidth',
    'winminheight',
    'winminwidth',
    'cmdheight',
    'laststatus',
    'showtabline',
    'lines',
    'columns',
    'previewheight',
    'helpheight',
    'switchbuf',
    'scrolloff',
    'hidden',
  }
  local got = {}
  for _, n in ipairs(names) do
    local ok, v = pcall(vim.api.nvim_get_option_value, n, {})
    -- `ok and v or 'ERR'` is the classic Lua trap: every `false`
    -- option value would read ERR.  Branch on `ok` explicitly.
    if ok then
      got[n] = v
    else
      got[n] = 'ERR'
    end
  end
  ask('s0/opt/global', 'defaults', { opts = got })
  local wnames = { 'winfixheight', 'winfixwidth', 'winfixbuf', 'previewwindow', 'winbar' }
  local gw = {}
  for _, n in ipairs(wnames) do
    local ok, v = pcall(vim.api.nvim_get_option_value, n, { win = 0 })
    if ok then
      gw[n] = v
    else
      gw[n] = 'ERR'
    end
  end
  ask('s0/opt/window', 'defaults', { opts = gw })
  ask('s0/layout/startup', 'before any option is pinned', snap())
  reset('one')
  run('s0/layout/pinned', {})
  for _, lay in ipairs(LAYNAMES) do
    reset(lay)
    run('s0/layout/' .. lay, {})
  end
end)

-- ============================================================ s1 wincmd

section('s1-wincmd', function()
  -- Every letter `do_window` dispatches on.  `q` and `c` can take the
  -- LAST window with them, which would end the sweep rather than
  -- produce a row, so they are only issued where a window survives.
  local LETTERS = {
    'x', 'r', 'R', 'H', 'J', 'K', 'L', 'T', '=', '+', '-', '<', '>',
    '_', '|', 's', 'v', 'n', '^', 'o', 'w', 'W', 'p', 't', 'b', 'j',
    'k', 'h', 'l', 'z', 'P', 'i', 'd', '}', 'g', 'f', 'F', 'ê',
  }
  local CLOSERS = { 'q', 'c' }
  for _, lay in ipairs(LAYNAMES) do
    for _, letter in ipairs(LETTERS) do
      reset(lay)
      focus(2)
      run(string.format('s1/%s/%s', lay, esc(letter)), 'wincmd ' .. letter)
    end
    for _, letter in ipairs(CLOSERS) do
      reset(lay)
      focus(2)
      local label = string.format('s1/%s/%s', lay, letter)
      if vim.fn.winnr('$') > 1 then
        run(label, 'wincmd ' .. letter)
      else
        -- One window and one tab: `:quit` ENDS the process.  The guard
        -- is the row.
        ask(label, 'guarded: last window', snap())
      end
    end
  end

  -- Counts.  `do_window` reads `Prenum` for the sizing letters and as a
  -- window NUMBER for `w`/`W`/`t`/`b`/`gt`, and 0 means "no count",
  -- which is a different arm from 1.
  local COUNTED = {
    '+', '-', '<', '>', '_', '|', 'w', 'W', 'x', 'r', 'j', 'k', 'h', 'l', 'n', 's', 'v',
  }
  for _, lay in ipairs({ 'col3', 'row3', 'grid4', 'nest' }) do
    for _, letter in ipairs(COUNTED) do
      for _, n in ipairs({ 0, 1, 2, 3, 5, 99 }) do
        reset(lay)
        focus(2)
        local cmd = (n == 0) and ('wincmd ' .. letter)
          or (tostring(n) .. 'wincmd ' .. letter)
        run(
          string.format('s1c/%s/%s/%d', lay, esc(letter), n),
          cmd
        )
      end
    end
  end

  -- `wincmd g<char>` -- the second-character dispatch.
  for _, nchar in ipairs({ 't', 'T', '}', ']', 'f', 'F', 'x' }) do
    reset('grid4')
    focus(2)
    run('s1g/grid4/g' .. nchar, 'wincmd g' .. nchar)
  end

  -- `wincmd ^` succeeds only with an alternate FILE; give it one.
  reset('h2')
  run('s1alt/hat', { 'silent! edit ' .. work .. '/alpha.txt', 'silent! edit ' .. work .. '/beta.txt', 'wincmd ^' })
  reset('h2')
  run('s1alt/hat-count', { 'silent! edit ' .. work .. '/alpha.txt', 'silent! edit ' .. work .. '/beta.txt', '1wincmd ^' })

  -- The preview window: `wincmd P`/`z` have a target now.
  reset('h2')
  run('s1prev/open', { 'silent! pedit ' .. work .. '/alpha.txt', 'wincmd P' })
  reset('h2')
  run('s1prev/close', { 'silent! pedit ' .. work .. '/alpha.txt', 'wincmd P', 'wincmd z' })
  reset('h2')
  run('s1prev/height', { 'set previewheight=5', 'silent! pedit ' .. work .. '/alpha.txt' })

  -- Rotation and exchange are the two letters with no coverage
  -- anywhere; walk them repeatedly so a fixed point shows.
  for _, letter in ipairs({ 'r', 'R', 'x' }) do
    reset('col3')
    for i = 1, 4 do
      run(string.format('s1rot/col3/%s/%d', letter, i), 'wincmd ' .. letter)
    end
    reset('row3')
    for i = 1, 4 do
      run(string.format('s1rot/row3/%s/%d', letter, i), 'wincmd ' .. letter)
    end
  end

  -- `H J K L T` move the window to an edge / a new tab; do each from
  -- every window of a 2x2 grid, because `winframe_remove` +
  -- `win_split_ins` take a different arm per position.
  for _, letter in ipairs({ 'H', 'J', 'K', 'L', 'T' }) do
    for n = 1, 4 do
      reset('grid4')
      focus(n)
      run(string.format('s1move/grid4/%s/%d', letter, n), 'wincmd ' .. letter)
    end
  end
end)

-- ============================================================= s2 split

section('s2-split', function()
  local MODS = {
    ['plain'] = '',
    ['topleft'] = 'topleft ',
    ['botright'] = 'botright ',
    ['leftabove'] = 'leftabove ',
    ['rightbelow'] = 'rightbelow ',
    ['aboveleft'] = 'aboveleft ',
    ['belowright'] = 'belowright ',
    ['vertical'] = 'vertical ',
  }
  local MODNAMES = {
    'plain', 'topleft', 'botright', 'leftabove', 'rightbelow',
    'aboveleft', 'belowright', 'vertical',
  }
  local CMDS = { 'split', 'vsplit', 'new', 'vnew' }

  for _, sb in ipairs({ 'nosplitbelow', 'splitbelow' }) do
    for _, sr in ipairs({ 'nosplitright', 'splitright' }) do
      for _, ea in ipairs({ 'noequalalways', 'equalalways' }) do
        for _, lay in ipairs({ 'one', 'grid4' }) do
          for _, cmd in ipairs(CMDS) do
            for _, mod in ipairs(MODNAMES) do
              reset(lay, { 'set ' .. sb, 'set ' .. sr, 'set ' .. ea })
              focus(2)
              run(
                string.format(
                  's2/%s/%s/%s/%s/%s/%s',
                  sb, sr, ea, lay, cmd, mod
                ),
                MODS[mod] .. cmd
              )
            end
          end
        end
      end
    end
  end

  -- Counts on the split commands: `:5split` is `frame_new_height`'s
  -- explicit-size arm, which never runs from a plain `:split`.
  for _, cmd in ipairs({ 'split', 'vsplit', 'new', 'vnew' }) do
    for _, n in ipairs({ 1, 3, 8, 20, 60, 200 }) do
      reset('h2')
      focus(1)
      run(string.format('s2n/%s/%d', cmd, n), tostring(n) .. cmd)
      reset('h2')
      focus(1)
      run(
        string.format('s2n/vertical-%s/%d', cmd, n),
        'vertical ' .. tostring(n) .. cmd
      )
    end
  end

  -- close / only / hide / quit, from every window of every layout.
  for _, lay in ipairs({ 'h2', 'v2', 'col3', 'row3', 'grid4', 'nest' }) do
    for n = 1, 4 do
      for _, cmd in ipairs({ 'close', 'close!', 'only', 'only!', 'hide', 'quit' }) do
        reset(lay, { 'set equalalways' })
        focus(n)
        local before = vim.fn.winnr('$')
        local label = string.format('s2c/%s/%d/%s', lay, n, cmd:gsub('!', 'bang'))
        if before > 1 then
          run(label, cmd)
        else
          ask(label, 'guarded: last window', snap())
        end
      end
    end
  end

  -- `winframe_remove` hands the closed window's rows to a neighbour and
  -- tells `frame_new_height` whether to grow that neighbour from its
  -- TOP (`altfr == frp_close->fr_next`).  For a LEAF neighbour the flag
  -- is dead -- `frame_new_height` only reads it in its FR_COL arm -- so
  -- every layout above is blind to it: a column's siblings are leaves
  -- or ROW frames, and the ROW arm merely passes it down.  It takes a
  -- three-deep nest (`C[R[C[...] w] w]`) before the flag reaches a
  -- column that has to choose an end, which is what `deep` is.
  for n = 1, 6 do
    for _, cmd in ipairs({ 'close', 'only', 'hide', 'quit', 'wincmd c' }) do
      reset('deep', { 'set equalalways' })
      focus(n)
      local label = string.format('s2cd/deep/%d/%s', n, cmd:gsub(' ', '_'))
      if vim.fn.winnr('$') > 1 then
        run(label, cmd)
      else
        ask(label, 'guarded: last window', snap())
      end
    end
  end
  for n = 1, 6 do
    reset('deep', { 'set noequalalways' })
    focus(n)
    run(string.format('s2cd/deep-noea/%d/close', n), 'close')
  end

  -- `:close` on the last window of a tab, and `:only` with a float up
  -- (which it must NOT close -- b19-2's rule, re-asserted here).
  reset('h2')
  run('s2c/tab/lastwin', { 'tabnew', 'close' })
  reset('h2')
  run('s2c/tab/onlywin-close', { 'tabnew', 'only', 'close' })
  reset('h2')
  local fb = POOL[3]
  do
    local ok = pcall(
      vim.api.nvim_open_win,
      fb,
      false,
      { relative = 'editor', row = 3, col = 3, width = 10, height = 4 }
    )
    run('s2c/float/only', ok and 'only' or 'nofloat')
  end
  reset('h2')
  pcall(
    vim.api.nvim_open_win,
    fb,
    false,
    { relative = 'editor', row = 3, col = 3, width = 10, height = 4 }
  )
  run('s2c/float/wincmd-o', 'wincmd o')

  -- `:new` and `:split` with a file name, so `win_split_ins` runs with
  -- a buffer swap behind it.
  for _, cmd in ipairs({ 'split', 'new', 'vsplit', 'vnew' }) do
    reset('one')
    run('s2f/' .. cmd, cmd .. ' ' .. work .. '/alpha.txt')
  end

  -- The room errors: `E36`/`E23` come out of `win_split_ins`'s
  -- `available < needed` guard, and nothing else reaches them.
  reset('one', { 'set winminheight=1' })
  local cmds = {}
  for _ = 1, 30 do
    cmds[#cmds + 1] = 'split'
  end
  run('s2room/hsplit-30', cmds)
  reset('one', { 'set winminwidth=1' })
  cmds = {}
  for _ = 1, 45 do
    cmds[#cmds + 1] = 'vsplit'
  end
  run('s2room/vsplit-45', cmds)
  reset('one', { 'set winminheight=4' })
  cmds = {}
  for _ = 1, 8 do
    cmds[#cmds + 1] = 'split'
  end
  run('s2room/hsplit-wmh4', cmds)
end)

-- ============================================================ s3 resize

section('s3-resize', function()
  local RESIZES = {
    'resize 1', 'resize 3', 'resize 8', 'resize 15', 'resize 30',
    'resize +1', 'resize +5', 'resize +40', 'resize -1', 'resize -5',
    'resize -40', 'resize 0', 'resize',
    'vertical resize 1', 'vertical resize 10', 'vertical resize 40',
    'vertical resize 79', 'vertical resize +5', 'vertical resize -5',
    'vertical resize +200', 'vertical resize -200', 'vertical resize 0',
    'vertical resize',
  }
  for _, lay in ipairs({ 'col3', 'row3', 'grid4', 'nest' }) do
    for _, cmd in ipairs(RESIZES) do
      reset(lay)
      focus(2)
      run(string.format('s3r/%s/%s', lay, cmd:gsub('[ +]', '_')), cmd)
    end
  end

  -- `:{winnr}resize` addresses another window -- `win_setheight_win`
  -- with a `win` that is not `curwin`.
  for _, n in ipairs({ 1, 2, 3, 4, 9 }) do
    reset('grid4')
    focus(1)
    run(string.format('s3rn/grid4/%d', n), tostring(n) .. 'resize 5')
    reset('grid4')
    focus(1)
    run(
      string.format('s3rn/grid4/vert-%d', n),
      'vertical ' .. tostring(n) .. 'resize 20'
    )
  end

  -- The API pair, which bypasses the ex layer entirely.
  for _, n in ipairs({ 0, 1, 5, 20, 100 }) do
    reset('grid4')
    focus(2)
    local wins = vim.api.nvim_tabpage_list_wins(0)
    local ok, err = pcall(vim.api.nvim_win_set_height, wins[3], n)
    ask(
      string.format('s3api/height/%d', n),
      ok and 'ok' or errtext(err),
      snap()
    )
    reset('grid4')
    focus(2)
    wins = vim.api.nvim_tabpage_list_wins(0)
    ok, err = pcall(vim.api.nvim_win_set_width, wins[3], n)
    ask(
      string.format('s3api/width/%d', n),
      ok and 'ok' or errtext(err),
      snap()
    )
  end

  -- `winfixheight` / `winfixwidth`: the frame walk has to skip them,
  -- and `frame_fixed_height`/`frame_fixed_width` decide a whole subtree.
  for _, which in ipairs({ 'winfixheight', 'winfixwidth' }) do
    for n = 1, 4 do
      for _, cmd in ipairs({ 'resize 5', 'vertical resize 20', 'wincmd =', 'split', 'close' }) do
        reset('grid4')
        local wins = vim.api.nvim_tabpage_list_wins(0)
        if wins[n] then
          pcall(vim.api.nvim_set_option_value, which, true, { win = wins[n] })
        end
        focus(1)
        local label = string.format('s3fx/%s/%d/%s', which, n, cmd:gsub('[ =]', '_'))
        if cmd == 'close' and vim.fn.winnr('$') < 2 then
          ask(label, 'guarded', snap())
        else
          run(label, cmd)
        end
      end
    end
  end

  -- `winminheight` / `winminwidth`: `frame_minheight`/`frame_minwidth`
  -- are the clamp every resize passes through.
  for _, opt in ipairs({ 'winminheight', 'winminwidth' }) do
    for _, v in ipairs({ 0, 1, 2, 5, 10, 20 }) do
      reset('grid4', { 'set ' .. opt .. '=' .. v })
      focus(1)
      run(
        string.format('s3min/%s/%d', opt, v),
        { 'resize 1', 'vertical resize 1', 'wincmd =' }
      )
      reset('col3', { 'set ' .. opt .. '=' .. v })
      focus(1)
      run(string.format('s3min/%s/%d/split', opt, v), 'split')
    end
  end

  -- `frame_minheight`/`frame_minwidth` reserve ONE line/column for the
  -- current window when the `winmin*` option is 0 -- but only on the
  -- `next_curwin == NULL` path, and `win_equal` passes `NOWIN`
  -- (`(win_T *)-1`), not NULL.  The only callers that pass NULL are
  -- `frame_setheight`/`frame_setwidth` walking the SIBLINGS of the
  -- frame being resized, so the arm needs a resize of a window that is
  -- NOT the current one.  And it is only ever OBSERVABLE when the
  -- request exceeds the room, because the reservation is a term in the
  -- `width = room` clamp and nowhere else: the same case at `resize 12`
  -- answers identically with the bump and without it.  Both halves --
  -- another window, and an impossible target -- were found with a grid
  -- probe (B20-1's method) after the plain s3min block above failed to
  -- catch the mutation.
  for _, lay in ipairs({ 'row3', 'nest' }) do
    for _, v in ipairs({ 0, 1 }) do
      for _, c in ipairs({ 1, 2 }) do
        for _, n in ipairs({ 1, 2, 3 }) do
          for _, t in ipairs({ 79, 200 }) do
            reset(lay, { 'set winminwidth=' .. v })
            focus(c)
            run(
              string.format('s3minx/w/%s/%d/%d/%d/%d', lay, v, c, n, t),
              'vertical ' .. tostring(n) .. 'resize ' .. tostring(t)
            )
          end
          reset(lay, { 'set winminheight=' .. v })
          focus(c)
          run(
            string.format('s3minx/h/%s/%d/%d/%d', lay, v, c, n),
            tostring(n) .. 'resize 99'
          )
        end
      end
    end
  end

  -- The `winfix*` RESERVATION boundary.  `frame_setwidth` drops the
  -- reservation when `room - curfrp->fr_width < room_reserved`, and
  -- `frame_setheight`'s twin spells the same test `<=` -- an upstream
  -- asymmetry a rewrite is very likely to 'tidy'.  Equality needs a
  -- shape a plain `winfixwidth` case never produces: the pinned window
  -- must be the one the take loop reaches FIRST (otherwise the near
  -- sibling absorbs the whole delta and the reservation is never
  -- consulted), and the OTHER sibling must sit at
  -- `frame_minwidth(pinned)` above its own minimum.  Concretely, in a
  -- row of three with the LAST window pinned, the first at width 2 and
  -- the middle shrinking.  Found with a grid probe over
  -- pinned-index x left width x right width x target.
  for _, fx in ipairs({ 1, 3 }) do
    for _, a in ipairs({ 1, 2, 3 }) do
      for _, b in ipairs({ 1, 3 }) do
        for _, t in ipairs({ 4, 10, 30 }) do
          reset('row3')
          local wins = vim.api.nvim_tabpage_list_wins(0)
          pcall(vim.api.nvim_exec2, 'silent! vertical 1resize ' .. a, { output = false })
          pcall(vim.api.nvim_exec2, 'silent! vertical 3resize ' .. b, { output = false })
          if wins[fx] then
            pcall(vim.api.nvim_set_option_value, 'winfixwidth', true, { win = wins[fx] })
          end
          focus(2)
          run(
            string.format('s3resv/w/%d/%d/%d/%d', fx, a, b, t),
            'vertical resize ' .. tostring(t)
          )
        end
      end
    end
  end
  for _, fx in ipairs({ 1, 3 }) do
    for _, a in ipairs({ 1, 2, 3 }) do
      for _, t in ipairs({ 3, 8 }) do
        reset('col3')
        local wins = vim.api.nvim_tabpage_list_wins(0)
        pcall(vim.api.nvim_exec2, 'silent! 1resize ' .. a, { output = false })
        pcall(vim.api.nvim_exec2, 'silent! 3resize 1', { output = false })
        if wins[fx] then
          pcall(vim.api.nvim_set_option_value, 'winfixheight', true, { win = wins[fx] })
        end
        focus(2)
        run(
          string.format('s3resv/h/%d/%d/%d', fx, a, t),
          'resize ' .. tostring(t)
        )
      end
    end
  end

  -- `winheight`/`winwidth` are applied on ENTER, by `win_enter_ext`.
  for _, opt in ipairs({ 'winheight', 'winwidth' }) do
    for _, v in ipairs({ 1, 5, 12, 40, 999 }) do
      reset('grid4', { 'set ' .. opt .. '=' .. v })
      focus(1)
      run(
        string.format('s3wh/%s/%d', opt, v),
        { 'wincmd j', 'wincmd l', 'wincmd k' }
      )
    end
  end

  -- `equalalways` x `eadirection` -- `win_equal_rec`'s two axes.
  for _, ea in ipairs({ 'noequalalways', 'equalalways' }) do
    for _, dir in ipairs({ 'both', 'ver', 'hor' }) do
      for _, lay in ipairs({ 'grid4', 'nest', 'deep' }) do
        reset(lay, { 'set ' .. ea, 'set eadirection=' .. dir })
        focus(2)
        run(
          string.format('s3eq/%s/%s/%s/=', ea, dir, lay),
          { 'resize 3', 'vertical resize 12', 'wincmd =' }
        )
        reset(lay, { 'set ' .. ea, 'set eadirection=' .. dir })
        focus(2)
        run(string.format('s3eq/%s/%s/%s/split', ea, dir, lay), 'split')
        reset(lay, { 'set ' .. ea, 'set eadirection=' .. dir })
        focus(2)
        local label = string.format('s3eq/%s/%s/%s/close', ea, dir, lay)
        if vim.fn.winnr('$') > 1 then
          run(label, 'close')
        else
          ask(label, 'guarded', snap())
        end
      end
    end
  end

  -- The screen-shape options.  `'cmdheight'` is the arm that is
  -- INVISIBLE under `-l`: `did_set_cmdheight` is guarded by
  -- `full_screen`, which `-l` leaves false.
  for _, lay in ipairs({ 'one', 'col3', 'grid4' }) do
    for _, v in ipairs({ 0, 1, 2, 3, 5, 12 }) do
      reset(lay)
      run(string.format('s3scr/cmdheight/%s/%d', lay, v), 'set cmdheight=' .. v)
    end
    for _, v in ipairs({ 0, 1, 2, 3 }) do
      reset(lay)
      run(string.format('s3scr/laststatus/%s/%d', lay, v), 'set laststatus=' .. v)
    end
    for _, v in ipairs({ 0, 1, 2 }) do
      reset(lay)
      run(string.format('s3scr/showtabline/%s/%d', lay, v), 'set showtabline=' .. v)
    end
    for _, v in ipairs({ 6, 12, 24, 40, 80 }) do
      reset(lay)
      run(string.format('s3scr/lines/%s/%d', lay, v), 'set lines=' .. v)
    end
    for _, v in ipairs({ 20, 40, 80, 120, 200 }) do
      reset(lay)
      run(string.format('s3scr/columns/%s/%d', lay, v), 'set columns=' .. v)
    end
  end

  -- `winbar` steals a row from inside the window, not from the frame.
  for _, lay in ipairs({ 'one', 'grid4' }) do
    reset(lay)
    run('s3bar/' .. lay, 'set winbar=WB')
    reset(lay)
    run('s3bar/' .. lay .. '/local', 'setlocal winbar=WB')
  end

  -- `win_size_save`/`win_size_restore` behind `:mksession`-free
  -- `winrestcmd()` round-trips: the string is meant to be executable.
  for _, lay in ipairs({ 'col3', 'row3', 'grid4', 'nest' }) do
    reset(lay)
    focus(1)
    vim.api.nvim_exec2('resize 4', { output = false })
    local rc = vim.fn.winrestcmd()
    reset(lay)
    run('s3rc/' .. lay, rc)
  end
end)

-- =========================================================== s4 tabpage

section('s4-tabpage', function()
  for _, cmd in ipairs({
    'tabnew',
    'tabnew ' .. '<F>',
    '0tabnew',
    '1tabnew',
    '$tabnew',
    'tab split',
    'tab vsplit',
    '-tabnew',
    '+tabnew',
  }) do
    reset('h2')
    run('s4new/' .. cmd:gsub('[ /]', '_'), (cmd:gsub('<F>', work .. '/alpha.txt')))
  end

  local function tabs(n)
    local cmds = {}
    for i = 2, n do
      cmds[#cmds + 1] = 'tabnew'
      cmds[#cmds + 1] = 'split'
    end
    return cmds
  end

  for _, at in ipairs({ 1, 2, 3, 4 }) do
    for _, cmd in ipairs({
      'tabclose', 'tabclose!', 'tabonly', 'tabonly!',
      'tabnext', 'tabprevious', 'tabfirst', 'tablast',
      'tabmove', 'tabmove 0', 'tabmove 1', 'tabmove 2', 'tabmove $',
      'tabmove +', 'tabmove -', 'tabmove +2', 'tabmove -2',
      '2tabnext', '3tabnext', '2tabclose', 'tabnext #',
    }) do
      reset('h2')
      pcall(vim.api.nvim_exec2, table.concat(tabs(4), ' | '), { output = false })
      pcall(vim.api.nvim_exec2, 'silent! tabnext ' .. at, { output = false })
      run(
        string.format('s4t/%d/%s', at, cmd:gsub('[ #$+!-]', function(c)
          return ({ [' '] = '_', ['#'] = 'alt', ['$'] = 'end', ['+'] = 'p', ['-'] = 'm', ['!'] = 'bang' })[c]
        end)),
        cmd
      )
    end
  end

  -- `wincmd T` moves a window into a new tab; `wincmd t`/`b` do not.
  for _, lay in ipairs({ 'h2', 'grid4', 'nest' }) do
    for n = 1, 3 do
      reset(lay)
      focus(n)
      run(string.format('s4T/%s/%d', lay, n), 'wincmd T')
    end
  end

  -- `gt`/`gT` with and without a count, through `wincmd g`.
  for _, k in ipairs({ 'gt', 'gT' }) do
    for _, n in ipairs({ 0, 1, 2, 3 }) do
      reset('h2')
      pcall(vim.api.nvim_exec2, table.concat(tabs(4), ' | '), { output = false })
      pcall(vim.api.nvim_exec2, 'silent! tabnext 2', { output = false })
      run(
        string.format('s4g/%s/%d', k, n),
        (n == 0 and 'wincmd ' .. k or tostring(n) .. 'wincmd ' .. k)
      )
    end
  end

  -- Crossing tabs by handle: `win_goto` / `goto_tabpage_win`.
  reset('h2')
  pcall(vim.api.nvim_exec2, table.concat(tabs(4), ' | '), { output = false })
  local allwins = {}
  for _, t in ipairs(vim.api.nvim_list_tabpages()) do
    for _, w in ipairs(vim.api.nvim_tabpage_list_wins(t)) do
      allwins[#allwins + 1] = w
    end
  end
  for i, w in ipairs(allwins) do
    local ok, err = pcall(vim.api.nvim_set_current_win, w)
    ask(
      string.format('s4goto/api/%d', i),
      ok and 'ok' or errtext(err),
      snap()
    )
  end
  for i, w in ipairs(allwins) do
    local ok, res = pcall(vim.fn.win_gotoid, w)
    ask(
      string.format('s4goto/fn/%d', i),
      ok and ('r=' .. tostring(res)) or errtext(res),
      snap()
    )
  end

  -- The last window of a tab, closed every way there is.
  for _, cmd in ipairs({ 'close', 'quit', 'only', 'wincmd c', 'wincmd q' }) do
    reset('one')
    pcall(vim.api.nvim_exec2, 'tabnew | tabnew', { output = false })
    run('s4last/' .. cmd:gsub(' ', '_'), cmd)
  end

  -- `tabpagewinnr` / `tabpagebuflist` / `gettabinfo`, normalised.
  reset('h2')
  pcall(vim.api.nvim_exec2, table.concat(tabs(3), ' | '), { output = false })
  local ti = {}
  for t = 1, vim.fn.tabpagenr('$') do
    ti[t] = {
      winnr = vim.fn.tabpagewinnr(t),
      last = vim.fn.tabpagewinnr(t, '$'),
      nbuf = #vim.fn.tabpagebuflist(t),
      lay = laystr((function()
        local l = vim.fn.winlayout(t)
        return l
      end)()):gsub('%d+', 'ID'),
    }
  end
  ask('s4info/tabs', 'per tab', { tabs = ti })
end)

-- ============================================================= s5 float

section('s5-float', function()
  local fb = nil
  local function newfloat(cfg, enter)
    local ok, res = pcall(vim.api.nvim_open_win, fb, enter or false, cfg)
    return ok, res
  end

  -- relative x anchor -- the matrix that is in NO existing oracle.
  for _, rel in ipairs({
    'editor', 'win', 'cursor', 'mouse', 'laststatus', 'tabline',
  }) do
    for _, anchor in ipairs({ 'NW', 'NE', 'SW', 'SE' }) do
      for _, pos in ipairs({ { 0, 0 }, { 5, 10 }, { 20, 70 }, { -3, -4 } }) do
        reset('grid4')
        fb = POOL[5]
        focus(2)
        local cfg = {
          relative = rel,
          anchor = anchor,
          row = pos[1],
          col = pos[2],
          width = 12,
          height = 5,
        }
        if rel == 'win' then
          cfg.win = vim.api.nvim_tabpage_list_wins(0)[3]
        end
        local ok, res = newfloat(cfg)
        ask(
          string.format('s5ra/%s/%s/%d_%d', rel, anchor, pos[1], pos[2]),
          ok and 'ok' or errtext(res),
          snap()
        )
      end
    end
  end

  -- zindex: the sort key `float_zindex_cmp` uses and the order
  -- `win_float_remove` closes in.
  for _, zs in ipairs({
    { 50, 50, 50 },
    { 10, 60, 30 },
    { 300, 200, 100 },
    { 1, 2, 3 },
    { 250, 250, 249 },
  }) do
    reset('h2')
    fb = POOL[5]
    local made = {}
    for i, z in ipairs(zs) do
      local ok, res = newfloat({
        relative = 'editor',
        row = i,
        col = i * 3,
        width = 8,
        height = 3,
        zindex = z,
      })
      made[i] = ok and 'ok' or errtext(res)
    end
    local key = table.concat(zs, '_')
    ask('s5z/' .. key, table.concat(made, ','), snap())
    -- A BARE `:fclose` CLOSES `line1` FLOATS, AND `line1` DEFAULTS TO
    -- THE CURSOR LINE.  `:fclose` is `EX_RANGE` + `ADDR_OTHER` and
    -- `ex_fclose` reads `eap->line1`, but `do_one_cmd`'s "default is 1,
    -- not cursor" fixup patches only `line2` -- so with this fixture's
    -- cursor on line 30 a bare `:fclose` closes THIRTY floats, i.e. all
    -- of them.  Filed upstream
    -- (`ex-docmd-fclose-count-defaults-to-cursor-line.md`); kept as one
    -- row per spread, with a `cursor1` twin that proves the dependence.
    -- The ORDERING question -- `float_zindex_cmp` sorts descending, so
    -- `:fclose` takes the topmost float first -- can only be asked with
    -- an explicit count, which is what `n1`/`n2`/`n3` are for.  Without
    -- them the whole section closes every float every time and the
    -- comparator gates nothing.
    run('s5z/' .. key .. '/fclose', 'fclose')
    for _, spec in ipairs({
      { '1fclose', 'n1', 30 },
      { '2fclose', 'n2', 30 },
      { '3fclose', 'n3', 30 },
      { 'fclose!', 'bang', 30 },
      { 'fclose', 'cursor1', 1 },
    }) do
      reset('h2')
      fb = POOL[5]
      vim.api.nvim_win_set_cursor(0, { spec[3], 0 })
      for i, z in ipairs(zs) do
        newfloat({
          relative = 'editor',
          row = i,
          col = i * 3,
          width = 8,
          height = 3,
          zindex = z,
        })
      end
      run('s5z/' .. key .. '/' .. spec[2], spec[1])
    end
  end

  -- borders, titles and footers.
  local BORDERS = {
    'none', 'single', 'double', 'rounded', 'solid', 'shadow', 'bold',
  }
  for _, bd in ipairs(BORDERS) do
    reset('one')
    fb = POOL[6]
    local ok, res = newfloat({
      relative = 'editor',
      row = 2,
      col = 2,
      width = 12,
      height = 4,
      border = bd,
    })
    ask('s5b/' .. bd, ok and 'ok' or errtext(res), snap())
  end
  for _, bd in ipairs({
    { { '1', '2', '3', '4', '5', '6', '7', '8' }, 'eight' },
    { { 'x' }, 'one' },
    { { 'a', 'b' }, 'two' },
    { { 'a', 'b', 'c', 'd' }, 'four' },
    { { { 'q', 'ErrorMsg' }, '-', '+', '|', '+', '-', '+', '|' }, 'hl' },
    { { '', '', '', '', '', '', '', '' }, 'empty' },
  }) do
    reset('one')
    fb = POOL[6]
    local ok, res = newfloat({
      relative = 'editor',
      row = 2,
      col = 2,
      width = 12,
      height = 4,
      border = bd[1],
    })
    ask('s5b/arr/' .. bd[2], ok and 'ok' or errtext(res), snap())
  end
  for _, tp in ipairs({ 'left', 'center', 'right', 'bogus' }) do
    for _, kind in ipairs({ 'title', 'footer' }) do
      reset('one')
      fb = POOL[6]
      local cfg = {
        relative = 'editor',
        row = 2,
        col = 2,
        width = 12,
        height = 4,
        border = 'single',
      }
      cfg[kind] = 'TT'
      cfg[kind .. '_pos'] = tp
      local ok, res = newfloat(cfg)
      ask(
        string.format('s5tf/%s/%s', kind, tp),
        ok and 'ok' or errtext(res),
        snap()
      )
    end
  end
  for _, spec in ipairs({
    { title = 'T', desc = 'title-noborder' },
    { title = { { 'a', 'ErrorMsg' }, { 'b' } }, border = 'single', desc = 'title-chunks' },
    { title = '', border = 'single', desc = 'title-empty' },
    { title = 'VERYLONGTITLEEXCEEDING', border = 'single', desc = 'title-long' },
    { footer = { { 'f' } }, border = 'double', desc = 'footer-chunks' },
    { title = 'T', footer = 'F', border = 'rounded', desc = 'both' },
  }) do
    reset('one')
    fb = POOL[6]
    local cfg = {
      relative = 'editor',
      row = 2,
      col = 2,
      width = 12,
      height = 4,
    }
    for k, v in pairs(spec) do
      if k ~= 'desc' then
        cfg[k] = v
      end
    end
    local ok, res = newfloat(cfg)
    ask('s5tf/' .. spec.desc, ok and 'ok' or errtext(res), snap())
  end

  -- bufpos: the win-relative anchor computed from a buffer position.
  for _, bp in ipairs({ { 0, 0 }, { 5, 3 }, { 29, 0 }, { 59, 9 }, { 100, 0 }, { -1, -1 } }) do
    reset('grid4')
    fb = POOL[7]
    focus(1)
    local parent = vim.api.nvim_tabpage_list_wins(0)[3]
    local ok, res = newfloat({
      relative = 'win',
      win = parent,
      bufpos = bp,
      width = 10,
      height = 3,
    })
    ask(
      string.format('s5bp/%d_%d', bp[1], bp[2]),
      ok and 'ok' or errtext(res),
      snap()
    )
  end
  -- bufpos plus an explicit offset, and with the parent scrolled.
  reset('grid4')
  fb = POOL[7]
  focus(3)
  vim.api.nvim_win_set_cursor(0, { 45, 0 })
  vim.api.nvim_exec2('normal! zt', { output = false })
  do
    local parent = vim.api.nvim_get_current_win()
    local ok, res = newfloat({
      relative = 'win',
      win = parent,
      bufpos = { 44, 2 },
      row = 1,
      col = 1,
      width = 10,
      height = 3,
    })
    ask('s5bp/scrolled', ok and 'ok' or errtext(res), snap())
  end

  -- The flags.
  for _, spec in ipairs({
    { style = 'minimal', desc = 'minimal' },
    { focusable = false, desc = 'unfocusable' },
    { hide = true, desc = 'hidden' },
    { fixed = true, desc = 'fixed' },
    { fixed = false, desc = 'unfixed' },
    { noautocmd = true, desc = 'noautocmd' },
    { mouse = false, desc = 'nomouse' },
    { zindex = 1, desc = 'z1' },
    { zindex = 32000, desc = 'zbig' },
    { width = 1, height = 1, desc = 'tiny' },
    { width = 200, height = 100, desc = 'huge' },
    { row = 100, col = 200, desc = 'offscreen' },
    { row = -20, col = -40, desc = 'negative' },
  }) do
    reset('h2')
    fb = POOL[5]
    local cfg = {
      relative = 'editor',
      row = 2,
      col = 2,
      width = 12,
      height = 4,
    }
    for k, v in pairs(spec) do
      if k ~= 'desc' then
        cfg[k] = v
      end
    end
    local ok, res = newfloat(cfg)
    ask('s5flag/' .. spec.desc, ok and 'ok' or errtext(res), snap())
  end

  -- `fixed` is only observable when the float would be pushed on
  -- screen: place it past the right edge and compare.
  for _, fx in ipairs({ true, false }) do
    for _, col in ipairs({ 70, 78, 100 }) do
      reset('one')
      fb = POOL[5]
      local ok, res = newfloat({
        relative = 'editor',
        row = 2,
        col = col,
        width = 20,
        height = 4,
        fixed = fx,
      })
      ask(
        string.format('s5fix/%s/%d', tostring(fx), col),
        ok and 'ok' or errtext(res),
        snap()
      )
    end
  end

  -- Entering a float, focus rules, and what closing the parent does.
  reset('grid4')
  fb = POOL[5]
  do
    local parent = vim.api.nvim_tabpage_list_wins(0)[3]
    local ok, res = pcall(vim.api.nvim_open_win, fb, true, {
      relative = 'win',
      win = parent,
      row = 1,
      col = 1,
      width = 10,
      height = 3,
    })
    ask('s5par/enter', ok and 'ok' or errtext(res), snap())
    run('s5par/close-parent', 'silent! ' .. '3close')
    run('s5par/wincmd-w', 'wincmd w')
    run('s5par/wincmd-p', 'wincmd p')
  end
  reset('grid4')
  fb = POOL[5]
  do
    local parent = vim.api.nvim_tabpage_list_wins(0)[3]
    pcall(vim.api.nvim_open_win, fb, false, {
      relative = 'win',
      win = parent,
      row = 1,
      col = 1,
      width = 10,
      height = 3,
    })
    ask('s5par/detached', 'before', snap())
    pcall(vim.api.nvim_win_close, parent, true)
    ask('s5par/detached/after', 'parent closed', snap())
  end

  -- Unfocusable floats and the navigation letters that must skip them.
  reset('h2')
  fb = POOL[5]
  pcall(vim.api.nvim_open_win, fb, false, {
    relative = 'editor',
    row = 1,
    col = 1,
    width = 10,
    height = 3,
    focusable = false,
  })
  for _, cmd in ipairs({ 'wincmd w', 'wincmd W', 'wincmd t', 'wincmd b', 'wincmd j', '3wincmd w' }) do
    run('s5nav/unfocusable/' .. cmd:gsub(' ', '_'), cmd)
  end
  reset('h2')
  fb = POOL[5]
  pcall(vim.api.nvim_open_win, fb, false, {
    relative = 'editor',
    row = 1,
    col = 1,
    width = 10,
    height = 3,
  })
  for _, cmd in ipairs({ 'wincmd w', 'wincmd W', 'wincmd t', 'wincmd b', 'wincmd j', '3wincmd w' }) do
    run('s5nav/focusable/' .. cmd:gsub(' ', '_'), cmd)
  end

  -- Floats in a second tab: `win_close_othertab` and the per-tab float
  -- list.
  reset('h2')
  fb = POOL[5]
  pcall(vim.api.nvim_exec2, 'tabnew', { output = false })
  pcall(vim.api.nvim_open_win, fb, false, {
    relative = 'editor',
    row = 1,
    col = 1,
    width = 10,
    height = 3,
  })
  ask('s5tab/made', 'float in tab 2', snap())
  run('s5tab/back', 'tabprevious')
  run('s5tab/close-other', 'tabclose 2')

  -- `laststatus`/`showtabline` move a `relative='laststatus'|'tabline'`
  -- float without anyone reconfiguring it (`win_reconfig_floats`).
  for _, rel in ipairs({ 'laststatus', 'tabline' }) do
    for _, opt in ipairs({ 'set laststatus=0', 'set laststatus=1', 'set laststatus=2', 'set laststatus=3', 'set showtabline=0', 'set showtabline=2' }) do
      reset('h2')
      fb = POOL[5]
      pcall(vim.api.nvim_open_win, fb, false, {
        relative = rel,
        row = 0,
        col = 0,
        width = 10,
        height = 3,
      })
      run(string.format('s5anchor/%s/%s', rel, opt:gsub('[ =]', '_')), opt)
    end
  end

  -- Errors.
  for _, spec in ipairs({
    { { relative = 'bogus', row = 1, col = 1, width = 5, height = 5 }, 'rel' },
    { { relative = 'editor', anchor = 'XX', row = 1, col = 1, width = 5, height = 5 }, 'anchor' },
    { { relative = 'editor', row = 1, col = 1, width = 0, height = 5 }, 'width0' },
    { { relative = 'editor', row = 1, col = 1, width = 5, height = 0 }, 'height0' },
    { { relative = 'editor', row = 1, col = 1, width = 5, height = 5, zindex = 0 }, 'zindex0' },
    { { relative = 'editor', row = 1, col = 1, width = 5, height = 5, zindex = -1 }, 'zindexneg' },
    { { relative = 'editor', row = 1, col = 1, width = 5, height = 5, external = true }, 'external' },
    { { relative = 'win', win = 99999, row = 1, col = 1, width = 5, height = 5 }, 'badwin' },
    { { relative = 'editor', row = 1, col = 1, width = 5, height = 5, border = { 'a', 'b', 'c' } }, 'border3' },
    { { relative = 'editor', row = 1, col = 1, width = 5, height = 5, title = 'T', title_pos = 'nope', border = 'single' }, 'titlepos' },
    { { relative = 'editor', row = 1, col = 1, width = 5, height = 5, style = 'nope' }, 'style' },
    { { split = 'left', relative = 'editor', row = 1, col = 1, width = 5, height = 5 }, 'splitconflict' },
    { { relative = 'editor', width = 5, height = 5 }, 'norowcol' },
    { { relative = 'win', bufpos = { 1 }, width = 5, height = 5 }, 'bufpos1' },
  }) do
    reset('one')
    fb = POOL[5]
    local ok, res = newfloat(spec[1])
    ask('s5err/' .. spec[2], ok and 'ok' or errtext(res), snap())
  end
end)

-- ============================================================ s6 config

section('s6-config', function()
  local fb = POOL[4]
  local function withfloat(cfg)
    reset('grid4')
    focus(1)
    local base = {
      relative = 'editor',
      row = 3,
      col = 5,
      width = 12,
      height = 4,
    }
    for k, v in pairs(cfg or {}) do
      base[k] = v
    end
    local ok, w = pcall(vim.api.nvim_open_win, fb, false, base)
    return ok and w or nil
  end

  local MOVES = {
    { { row = 0, col = 0 }, 'origin' },
    { { row = 10, col = 40 }, 'mid' },
    { { row = 30, col = 100 }, 'past' },
    { { row = -5, col = -5 }, 'neg' },
    { { width = 1, height = 1 }, 'tiny' },
    { { width = 79, height = 22 }, 'big' },
    { { width = 300, height = 300 }, 'huge' },
    { { anchor = 'NE' }, 'anchor-ne' },
    { { anchor = 'SW' }, 'anchor-sw' },
    { { anchor = 'SE' }, 'anchor-se' },
    { { zindex = 300 }, 'z300' },
    { { zindex = 1 }, 'z1' },
    { { focusable = false }, 'unfocus' },
    { { hide = true }, 'hide' },
    { { border = 'double' }, 'border' },
    { { border = 'none' }, 'noborder' },
    { { title = 'X', title_pos = 'center' }, 'title' },
    { { footer = 'Y' }, 'footer' },
    { { style = 'minimal' }, 'minimal' },
    { { fixed = true }, 'fixed' },
    { { mouse = false }, 'nomouse' },
    { { relative = 'cursor', row = 1, col = 1 }, 'to-cursor' },
    { { relative = 'laststatus', row = 0, col = 0 }, 'to-laststatus' },
    { { relative = 'tabline', row = 0, col = 0 }, 'to-tabline' },
    { { relative = 'mouse', row = 0, col = 0 }, 'to-mouse' },
  }
  for _, m in ipairs(MOVES) do
    local w = withfloat()
    if w then
      local ok, err = pcall(vim.api.nvim_win_set_config, w, m[1])
      ask('s6mv/' .. m[2], ok and 'ok' or errtext(err), snap())
    end
  end

  -- Reparent: point the float at a different window, then at one in
  -- another tab, then at itself.
  do
    local w = withfloat({ relative = 'win', win = 0 })
    if w then
      local wins = vim.api.nvim_tabpage_list_wins(0)
      for i = 1, 4 do
        if wins[i] then
          local ok, err =
            pcall(vim.api.nvim_win_set_config, w, { relative = 'win', win = wins[i], row = 1, col = 1 })
          ask(
            string.format('s6re/parent/%d', i),
            ok and 'ok' or errtext(err),
            snap()
          )
        end
      end
      local ok, err =
        pcall(vim.api.nvim_win_set_config, w, { relative = 'win', win = w, row = 1, col = 1 })
      ask('s6re/parent/self', ok and 'ok' or errtext(err), snap())
    end
  end

  -- float <-> split conversion, both directions.
  for _, sp in ipairs({ 'left', 'right', 'above', 'below' }) do
    local w = withfloat()
    if w then
      local wins = vim.api.nvim_tabpage_list_wins(0)
      local ok, err = pcall(
        vim.api.nvim_win_set_config,
        w,
        { split = sp, win = wins[1] }
      )
      ask('s6conv/float2split/' .. sp, ok and 'ok' or errtext(err), snap())
    end
  end
  for _, sp in ipairs({ 'left', 'right', 'above', 'below' }) do
    reset('grid4')
    focus(2)
    local target = vim.api.nvim_tabpage_list_wins(0)[3]
    local ok, err = pcall(
      vim.api.nvim_win_set_config,
      target,
      { split = sp, win = vim.api.nvim_tabpage_list_wins(0)[1] }
    )
    ask('s6conv/split2split/' .. sp, ok and 'ok' or errtext(err), snap())
  end
  for _, sp in ipairs({ 'left', 'below' }) do
    reset('grid4')
    focus(2)
    local target = vim.api.nvim_tabpage_list_wins(0)[3]
    local ok, err = pcall(vim.api.nvim_win_set_config, target, {
      relative = 'editor',
      row = 2,
      col = 2,
      width = 10,
      height = 3,
    })
    ask('s6conv/split2float/' .. sp, ok and 'ok' or errtext(err), snap())
  end
  do
    reset('grid4')
    focus(2)
    local target = vim.api.nvim_tabpage_list_wins(0)[3]
    local ok, err = pcall(vim.api.nvim_win_set_config, target, {
      vertical = true,
      win = vim.api.nvim_tabpage_list_wins(0)[1],
    })
    ask('s6conv/vertical', ok and 'ok' or errtext(err), snap())
  end

  -- Setting a config on a NORMAL window: only `split`/`win`/`width`/
  -- `height` are legal there.
  for _, spec in ipairs({
    { { width = 30 }, 'width' },
    { { height = 6 }, 'height' },
    { { relative = '' }, 'relempty' },
    { { anchor = 'SE' }, 'anchor' },
    { { zindex = 100 }, 'zindex' },
    { { border = 'single' }, 'border' },
    { { hide = true }, 'hide' },
    { { focusable = false }, 'focusable' },
  }) do
    reset('grid4')
    focus(2)
    local ok, err = pcall(vim.api.nvim_win_set_config, 0, spec[1])
    ask('s6norm/' .. spec[2], ok and 'ok' or errtext(err), snap())
  end

  -- Repeated reconfigure of the same float -- `merge_win_config` keeps
  -- fields the caller omitted, and that is easy to lose.
  do
    local w = withfloat({ border = 'single', title = 'T', zindex = 70 })
    if w then
      for i, step in ipairs({
        { row = 1 },
        { col = 20 },
        { width = 20 },
        { height = 8 },
        { anchor = 'SE' },
        { relative = 'editor', row = 4, col = 4 },
      }) do
        local ok, err = pcall(vim.api.nvim_win_set_config, w, step)
        ask(
          string.format('s6merge/%d', i),
          ok and 'ok' or errtext(err),
          snap()
        )
      end
    end
  end

  -- hide/show cycles: a hidden float keeps its slot in the window list.
  do
    local w = withfloat()
    if w then
      for i, h in ipairs({ true, false, true, true, false }) do
        local ok, err = pcall(vim.api.nvim_win_set_config, w, { hide = h })
        ask(
          string.format('s6hide/%d/%s', i, tostring(h)),
          ok and 'ok' or errtext(err),
          snap()
        )
      end
    end
  end

  -- Closing floats every way there is.
  for _, how in ipairs({ 'api', 'apiforce', 'close', 'quit', 'fclose', 'fclosebang' }) do
    local w = withfloat()
    if w then
      local note
      if how == 'api' then
        local ok, err = pcall(vim.api.nvim_win_close, w, false)
        note = ok and 'ok' or errtext(err)
      elseif how == 'apiforce' then
        local ok, err = pcall(vim.api.nvim_win_close, w, true)
        note = ok and 'ok' or errtext(err)
      else
        pcall(vim.api.nvim_set_current_win, w)
        local cmd = ({ close = 'close', quit = 'quit', fclose = 'fclose', fclosebang = 'fclose!' })[how]
        local ok, res = pcall(vim.api.nvim_exec2, cmd, { output = true })
        note = ok and ((res.output or ''):gsub('\r?\n', ' | ')) or errtext(res)
      end
      ask('s6close/' .. how, note, snap())
    end
  end
end)

-- =========================================================== s7 auorder

section('s7-auorder', function()
  local child = child_start()
  local ok = child:lua(CPRELUDE .. '\nreturn 1')
  if ok ~= 1 then
    emit('s7/child', '!', 'PRELUDE ' .. esc(tostring(ok)))
    child:stop()
    return
  end
  child:lua([[
    _G.AUEVENTS = {'WinNewPre','WinNew','WinClosed','WinEnter','WinLeave',
                   'WinResized','WinScrolled','TabNew','TabNewEntered',
                   'TabEnter','TabLeave','TabClosed','BufWinEnter',
                   'BufWinLeave','BufEnter','BufLeave'}
    vim.api.nvim_create_augroup('wnau', {clear = true})
    for _, ev in ipairs(_G.AUEVENTS) do
      vim.api.nvim_create_autocmd(ev, {group = 'wnau', pattern = '*', callback = function(a)
        local log = vim.g.wnlog or {}
        local n = #log
        local ev2 = {}
        local idev = (ev == 'WinScrolled' or ev == 'WinResized' or ev == 'WinClosed')
        if ev == 'WinScrolled' or ev == 'WinResized' then
          local e = vim.v.event or {}
          local keys = {}
          for k in pairs(e) do keys[#keys+1] = k end
          table.sort(keys)
          local named = {}
          for _, k in ipairs(keys) do
            local v = e[k]
            local kn = (k == 'all' or k == 'windows') and k or _G.WID(k)
            if type(v) == 'table' then
              local kk = {}
              for k2 in pairs(v) do kk[#kk+1] = k2 end
              table.sort(kk)
              local ps = {}
              for _, k2 in ipairs(kk) do
                local vv = v[k2]
                if k == 'windows' then vv = _G.WID(vv) end
                ps[#ps+1] = k2..'='..tostring(vv)
              end
              named[#named+1] = {kn, kn..'{'..table.concat(ps, ',')..'}'}
            else
              named[#named+1] = {kn, kn..'='..tostring(v)}
            end
          end
          -- Re-sort on the MAPPED key: raw ids sort by allocation order
          -- and the ordinals do not, so an unsorted list would flip
          -- whenever a gesture above allocated a different number.
          table.sort(named, function(x, y) return x[1] < y[1] end)
          for _, pair in ipairs(named) do ev2[#ev2+1] = pair[2] end
        end
        local m = tostring(a.match)
        if idev then m = _G.WID(m) end
        log[n+1] = string.format('%s:%s:%s', ev, m, table.concat(ev2, ' '))
        vim.g.wnlog = log
        return false
      end})
    end
    return 1
  ]])

  --- One gesture: reset, clear the log, run it, then TURN THE MAIN LOOP
  --- once -- `WinResized`/`WinScrolled` fire from `normal_check` and
  --- from nowhere a script can reach.
  local function gesture(label, pre, lay, code)
    child:lua(
      string.format(
        'RESET(%s, %s); vim.g.wnlog = {}; return 1',
        tolua(pre or {}),
        tolua(lay or {})
      )
    )
    child:turn()
    child:lua('REMAP(); vim.g.wnlog = {}; return 1')
    local res = child:lua(
      'local ok, err = pcall(function() ' .. code .. ' end); return ok and "ok" or tostring(err)'
    )
    child:turn()
    child:turn()
    local log = child:lua('return vim.g.wnlog')
    local counts = {}
    if type(log) == 'table' then
      for _, line in ipairs(log) do
        local ev = line:match('^([^:]+)')
        counts[ev] = (counts[ev] or 0) + 1
      end
    end
    label_once(label)
    local seq = {}
    if type(log) == 'table' then
      for i, line in ipairs(log) do
        seq[i] = line
      end
    end
    emit(
      label,
      '=',
      esc(scrub(tostring(res))),
      esc(scrub(cap(table.concat(seq, ' > '), 700)))
    )
    struct(label, { res = res, seq = seq, counts = counts })
  end

  local BASE2 = { 'set laststatus=2 lines=24 columns=80' }
  gesture('s7/split', BASE2, {}, "vim.cmd('split')")
  gesture('s7/vsplit', BASE2, {}, "vim.cmd('vsplit')")
  gesture('s7/new', BASE2, {}, "vim.cmd('new')")
  gesture('s7/close', BASE2, { 'split' }, "vim.cmd('close')")
  gesture('s7/only', BASE2, { 'split', 'vsplit' }, "vim.cmd('only')")
  gesture('s7/quit', BASE2, { 'split' }, "vim.cmd('quit')")
  gesture('s7/hide', BASE2, { 'split' }, "vim.cmd('hide')")
  gesture('s7/resize', BASE2, { 'split' }, "vim.cmd('resize 5')")
  gesture('s7/vresize', BASE2, { 'vsplit' }, "vim.cmd('vertical resize 20')")
  gesture('s7/equal', BASE2, { 'split', 'vsplit' }, "vim.cmd('resize 3'); vim.cmd('wincmd =')")
  gesture('s7/wincmd-w', BASE2, { 'split' }, "vim.cmd('wincmd w')")
  gesture('s7/wincmd-x', BASE2, { 'split' }, "vim.cmd('wincmd x')")
  gesture('s7/wincmd-J', BASE2, { 'vsplit' }, "vim.cmd('wincmd J')")
  gesture('s7/wincmd-T', BASE2, { 'split' }, "vim.cmd('wincmd T')")
  gesture('s7/tabnew', BASE2, {}, "vim.cmd('tabnew')")
  gesture('s7/tabclose', BASE2, { 'tabnew' }, "vim.cmd('tabclose')")
  gesture('s7/tabonly', BASE2, { 'tabnew', 'tabnew' }, "vim.cmd('tabonly')")
  gesture('s7/tabnext', BASE2, { 'tabnew', 'tabnew' }, "vim.cmd('tabnext 1')")
  gesture('s7/tabmove', BASE2, { 'tabnew', 'tabnew' }, "vim.cmd('tabmove 0')")
  gesture(
    's7/float-open',
    BASE2,
    {},
    "local b = vim.api.nvim_create_buf(false, true); vim.api.nvim_open_win(b, false, {relative='editor', row=1, col=1, width=8, height=3})"
  )
  gesture(
    's7/float-enter',
    BASE2,
    {},
    "local b = vim.api.nvim_create_buf(false, true); vim.api.nvim_open_win(b, true, {relative='editor', row=1, col=1, width=8, height=3})"
  )
  gesture(
    's7/float-noautocmd',
    BASE2,
    {},
    "local b = vim.api.nvim_create_buf(false, true); vim.api.nvim_open_win(b, true, {relative='editor', row=1, col=1, width=8, height=3, noautocmd=true})"
  )
  gesture(
    's7/float-close',
    BASE2,
    {},
    "local b = vim.api.nvim_create_buf(false, true); local w = vim.api.nvim_open_win(b, false, {relative='editor', row=1, col=1, width=8, height=3}); vim.api.nvim_win_close(w, true)"
  )
  gesture(
    's7/setconfig',
    BASE2,
    {},
    "local b = vim.api.nvim_create_buf(false, true); local w = vim.api.nvim_open_win(b, false, {relative='editor', row=1, col=1, width=8, height=3}); vim.api.nvim_win_set_config(w, {width=20, height=8})"
  )
  gesture('s7/scroll', BASE2, { 'split' }, "vim.api.nvim_win_set_cursor(0, {40, 0})")
  gesture('s7/lines', BASE2, { 'split' }, "vim.o.lines = 40")
  gesture('s7/columns', BASE2, { 'split' }, "vim.o.columns = 120")
  gesture('s7/cmdheight', BASE2, { 'split' }, "vim.o.cmdheight = 4")
  gesture('s7/laststatus', BASE2, { 'split' }, "vim.o.laststatus = 0")
  gesture('s7/showtabline', BASE2, { 'split' }, "vim.o.showtabline = 2")
  gesture(
    's7/eventignore',
    { 'set laststatus=2 lines=24 columns=80', 'set eventignore=WinNew' },
    {},
    "vim.cmd('split')"
  )
  gesture(
    's7/nested-close',
    BASE2,
    { 'split' },
    "vim.api.nvim_create_autocmd('WinClosed', {once = true, callback = function() pcall(vim.cmd, 'split') end}); vim.cmd('close')"
  )
  gesture(
    's7/winnewpre-veto',
    BASE2,
    {},
    "vim.api.nvim_create_autocmd('WinNewPre', {once = true, callback = function() error('no') end}); pcall(vim.cmd, 'split')"
  )

  child:stop()
  emit('s7/child', '=', 'stderr=' .. esc(table.concat(said(child.err), ' | ')))
end)

-- ======================================================== s91 crashprobe

section('s91-crashprobe', function()
  -- Each of these gets its own child: the question is "does the editor
  -- survive", and a survivor answers while a corpse says so on stderr.
  local CASES = {
    { 'huge-resize', "vim.cmd('split'); vim.cmd('resize 2147483647')" },
    { 'huge-vresize', "vim.cmd('vsplit'); vim.cmd('vertical resize 2147483647')" },
    { 'neg-resize', "vim.cmd('split'); vim.cmd('resize -2147483647')" },
    { 'huge-count-plus', "vim.cmd('split'); vim.cmd('2147483647wincmd +')" },
    { 'huge-count-bar', "vim.cmd('vsplit'); vim.cmd('2147483647wincmd |')" },
    { 'huge-winminheight', "vim.cmd('set winminheight=2147483647')" },
    { 'huge-winheight', "vim.cmd('set winheight=2147483647'); vim.cmd('split')" },
    { 'huge-cmdheight', "vim.cmd('set cmdheight=2147483647')" },
    { 'lines-1', "vim.cmd('set lines=1')" },
    { 'columns-1', "vim.cmd('set columns=1')" },
    { 'many-splits', "for _ = 1, 400 do pcall(vim.cmd, 'split') end; return vim.fn.winnr('$')" },
    { 'many-vsplits', "for _ = 1, 400 do pcall(vim.cmd, 'vsplit') end; return vim.fn.winnr('$')" },
    { 'many-tabs', "for _ = 1, 400 do pcall(vim.cmd, 'tabnew') end; return vim.fn.tabpagenr('$')" },
    { 'deep-nest', "for i = 1, 60 do pcall(vim.cmd, i % 2 == 0 and 'split' or 'vsplit') end; return #vim.fn.winrestcmd()" },
    { 'float-huge', "local b = vim.api.nvim_create_buf(false, true); return tostring(pcall(vim.api.nvim_open_win, b, false, {relative='editor', row=2147483647, col=2147483647, width=2147483647, height=2147483647}))" },
    { 'float-zindex-max', "local b = vim.api.nvim_create_buf(false, true); return tostring(pcall(vim.api.nvim_open_win, b, false, {relative='editor', row=1, col=1, width=5, height=5, zindex=2147483647}))" },
    { 'float-bufpos-huge', "local b = vim.api.nvim_create_buf(false, true); return tostring(pcall(vim.api.nvim_open_win, b, false, {relative='win', win=0, bufpos={2147483647, 2147483647}, width=5, height=5}))" },
    { 'float-selfparent', "local b = vim.api.nvim_create_buf(false, true); local w = vim.api.nvim_open_win(b, false, {relative='editor', row=1, col=1, width=5, height=5}); return tostring(pcall(vim.api.nvim_win_set_config, w, {relative='win', win=w, row=1, col=1}))" },
    { 'float-close-in-au', "local b = vim.api.nvim_create_buf(false, true); local w = vim.api.nvim_open_win(b, false, {relative='editor', row=1, col=1, width=5, height=5}); vim.api.nvim_create_autocmd('WinClosed', {once=true, callback=function() pcall(vim.api.nvim_win_close, w, true) end}); pcall(vim.api.nvim_win_close, w, true); return vim.fn.winnr('$')" },
    { 'close-in-winenter', "vim.cmd('split'); vim.api.nvim_create_autocmd('WinEnter', {callback=function() pcall(vim.cmd, 'close') end}); pcall(vim.cmd, 'wincmd w'); return vim.fn.winnr('$')" },
    { 'tabclose-in-tableave', "vim.cmd('tabnew'); vim.api.nvim_create_autocmd('TabLeave', {callback=function() pcall(vim.cmd, 'tabclose') end}); pcall(vim.cmd, 'tabnext'); return vim.fn.tabpagenr('$')" },
    { 'winrestcmd-400', "for _ = 1, 200 do pcall(vim.cmd, 'split') end; local s = vim.fn.winrestcmd(); pcall(vim.cmd, s); return #s" },
    { 'equal-deep', "vim.cmd('set equalalways'); for i = 1, 80 do pcall(vim.cmd, i % 2 == 0 and 'split' or 'vsplit') end; pcall(vim.cmd, 'wincmd ='); return vim.fn.winnr('$')" },
    { 'move-all-J', "for _ = 1, 20 do pcall(vim.cmd, 'vsplit') end; for _ = 1, 20 do pcall(vim.cmd, 'wincmd J') end; return vim.fn.winnr('$')" },
  }
  local aborted = 0
  for _, case in ipairs(CASES) do
    local child = child_start()
    local pre = child:lua(CPRELUDE .. '\nBASE(); return 1')
    local res
    if pre == 1 then
      res = child:lua(
        'local ok, r = pcall(function() '
          .. case[2]
          .. ' end); return tostring(ok) .. "/" .. tostring(r)'
      )
    else
      res = 'PRELUDE ' .. tostring(pre)
    end
    local alive = child:lua('return vim.fn.winnr("$")')
    local dead = child.dead
    if dead then
      aborted = aborted + 1
    end
    child:stop()
    local words = said(child.err)
    label_once('s91/' .. case[1])
    emit(
      's91/' .. case[1],
      'X',
      'dead=' .. tostring(dead),
      'r=' .. esc(scrub(cap(tostring(res), 200))),
      'alive=' .. esc(tostring(alive)),
      'said=' .. esc(cap(table.concat(words, ' | '), 300))
    )
    struct('s91/' .. case[1], {
      dead = dead,
      res = tostring(res),
      alive = tostring(alive),
      said = words,
    })
  end
  emit(
    's91',
    'groups',
    string.format('cases=%d aborted=%d', #CASES, aborted)
  )
end)

emit('##', 'TOTAL', string.format('rows=%d', rows))
structfd:close()
