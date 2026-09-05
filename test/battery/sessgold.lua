-- sessgold -- the `:mksession` / `:mkview` / `:mkvimrc` / `:mkexrc` golden.
--
-- `:mksession` writes an ON-DISK FORMAT, and a session file that is subtly
-- wrong still sources back without a murmur -- which is why 24 of
-- test_mksession.vim's 50 functions cannot see a regression that changes what
-- was written but not what sourcing it does.  cmdsweep's s17s reads the file
-- back but normalises `\d\+` to `N`, i.e. it scrubs *every digit*, and window
-- sizes, `exe '1resize ' . ((&lines * 11 + 12) / 24)`, `badd +N`, `normal! 016|`
-- and the cursor line/column are exactly what makeopens (453 lines) and
-- put_view (230) exist to emit.  So this oracle keeps the bytes.
--
-- One child per scene.  The scene is built, then every `:mk*` form is run
-- against it and the emitted file is reported verbatim, one `F` line per file
-- line, with only two things scrubbed:
--
--   * the work directory -- in its plain spelling, in its `=+`-encoded
--     spelling (get_view_file() maps '/' -> '=+' and '=' -> '=='), and in
--     $VIMRUNTIME / $HOME form;
--   * `v:this_session`, which is that same path.
--
-- Digits carry.  A scrub that eats a field must eat its separator too, or the
-- field's *width* leaks into the artifact (B16-18's `%-10s` pid padding); the
-- work-directory scrub replaces whole path tokens, so two work directories of
-- different lengths produce byte-identical artifacts.  That is what the
-- two-directory determinism run proves.
--
--   SG_WORK=... SG_RUNTIME=... SG_SCENE=splits nvim -l sessgold.lua
--   SG_SCENE=LIST ... -> the scene names, one per line

local out = io.stdout
out:setvbuf('line')

local api = vim.api
local fn = vim.fn

local WORK = os.getenv('SG_WORK') or '/tmp/sessgold'
local RUNTIME = os.getenv('SG_RUNTIME') or (os.getenv('VIMRUNTIME') or '')
local HOME = os.getenv('HOME') or ''
local SCENE = os.getenv('SG_SCENE') or ''

--------------------------------------------------------------------------
-- Scrub + escape
--------------------------------------------------------------------------

local function lit(s) return (s:gsub('[%^%$%(%)%%%.%[%]%*%+%-%?]', '%%%1')) end

-- get_view_file()'s encoding of a path into a single file name.
local function vencode(p) return (p:gsub('=', '=='):gsub('/', '=+')) end

local SUBS = {
  { lit(vencode(WORK)), '<WORK>' },
  { lit(WORK), '<WORK>' },
}
if RUNTIME ~= '' then
  SUBS[#SUBS + 1] = { lit(vencode(RUNTIME)), '<RT>' }
  SUBS[#SUBS + 1] = { lit(RUNTIME), '<RT>' }
end
if HOME ~= '' and HOME ~= '/' then
  SUBS[#SUBS + 1] = { lit(HOME), '<HOME>' }
end

local function scrub(s)
  s = tostring(s)
  for _, p in ipairs(SUBS) do s = s:gsub(p[1], p[2]) end
  -- A pcall'd `vim.cmd` error is prefixed with THIS file's path and the line
  -- the call sits on, so every recorded error would re-baseline on an edit
  -- anywhere above it (B16-5's Lua-traceback trap, in its shortest form).
  s = s:gsub('^[^\n]-sessgold%.lua:%d+: ', '')
  return s
end

-- Keep the artifact one-line-per-record and greppable.  Only C0 and DEL are
-- escaped: :mkvimrc emits raw <C-W>/<Esc> inside mappings, and a raw newline
-- there would forge a record boundary.  High bytes are UTF-8 and pass through.
local function esc(s)
  s = s:gsub('\\', '\\\\')
  s = s:gsub('[%z\1-\31\127]', function(c) return ('\\x%02x'):format(c:byte()) end)
  return s
end

local function say(...) out:write(table.concat({ ... }, '\t'), '\n') end

--------------------------------------------------------------------------
-- Fixture files (written by the driver; names only here)
--------------------------------------------------------------------------

local A = WORK .. '/a.txt'
local B = WORK .. '/b.txt'
local C = WORK .. '/c.txt'
local D = WORK .. '/deep/d.txt'
local E = WORK .. '/odd name%#.txt'

--------------------------------------------------------------------------
-- Scenes
--------------------------------------------------------------------------

local function cmd(s) api.nvim_command(s) end

local SCENES = {}
local ORDER = {}
local function S(name, build)
  SCENES[name] = build
  ORDER[#ORDER + 1] = name
end

-- 1. One file, one window.  The floor: makeopens' preamble, one `argglobal`,
--    put_view's fold block and cursor restore.
S('plain', function()
  cmd('edit ' .. fn.fnameescape(A))
  cmd('setlocal textwidth=44 shiftwidth=3')
  fn.cursor(4, 6)
  cmd('normal! zt')
end)

-- 2. Splits.  ses_win_rec's frame walk, ses_winsizes' resize arithmetic and
--    the splitbelow/splitright save-restore pair.
S('splits', function()
  cmd('edit ' .. fn.fnameescape(A))
  cmd('split ' .. fn.fnameescape(B))
  cmd('vsplit ' .. fn.fnameescape(C))
  cmd('wincmd j')
  cmd('resize 4')
  cmd('wincmd k')
  cmd('vertical resize 31')
  cmd('setlocal nowrap number foldcolumn=2')
  fn.cursor(2, 3)
  cmd('wincmd j')
  fn.cursor(5, 1)
  cmd('wincmd t')
  fn.cursor(3, 4)
end)

-- 3. Tab pages.  `tabnew +setlocal\ bufhidden=wipe`, `tabrewind`, `set stal=2`,
--    and the per-tab frame walk.
S('tabs', function()
  cmd('edit ' .. fn.fnameescape(A))
  cmd('split ' .. fn.fnameescape(B))
  cmd('tabnew ' .. fn.fnameescape(C))
  cmd('tabnew ' .. fn.fnameescape(D))
  cmd('vsplit ' .. fn.fnameescape(A))
  cmd('vertical resize 22')
  cmd('tabnext 2')
  fn.cursor(1, 2)
end)

-- 4. Folds.  put_view's three fold spellings: manual folds are replayed as
--    `N,Mfold` + `Nfoldclose`, expr/indent/marker as `setlocal foldmethod=`.
S('folds', function()
  cmd('edit ' .. fn.fnameescape(A))
  cmd('setlocal foldmethod=manual foldlevel=1 foldminlines=0')
  cmd('2,4fold')
  cmd('6,8fold')
  cmd('normal! 2Gzc')
  cmd('split ' .. fn.fnameescape(B))
  cmd('setlocal foldmethod=indent foldnestmax=3 foldlevel=0 foldignore=;')
  cmd('split ' .. fn.fnameescape(C))
  cmd('setlocal foldmethod=expr foldexpr=len(getline(v:lnum))%2 foldenable')
  cmd('wincmd b')
  fn.cursor(9, 1)
end)

-- 5. Argument lists.  ses_arglist' quoting, `%argdel`, `argglobal` vs
--    `arglocal`, and `:argument N` restoring the index.
S('args', function()
  cmd('args ' .. fn.fnameescape(A) .. ' ' .. fn.fnameescape(B) .. ' ' .. fn.fnameescape(E))
  cmd('argument 2')
  cmd('split')
  cmd('arglocal')
  cmd('argdelete *')
  cmd('argadd ' .. fn.fnameescape(C) .. ' ' .. fn.fnameescape(D))
  cmd('argument 1')
  cmd('wincmd t')
end)

-- 6. Buffers.  `badd +N`, and the four buffers makeopens must decide about:
--    unlisted, help, a scratch `nofile`, and a no-name.
S('buffers', function()
  cmd('edit ' .. fn.fnameescape(A))
  cmd('badd +3 ' .. fn.fnameescape(B))
  cmd('badd +1 ' .. fn.fnameescape(E))
  cmd('edit ' .. fn.fnameescape(C))
  cmd('setlocal nobuflisted')
  cmd('enew')
  cmd('setlocal buftype=nofile bufhidden=hide noswapfile')
  api.nvim_buf_set_lines(0, 0, -1, false, { 'scratch one', 'scratch two' })
  cmd('help help.txt')
  cmd('wincmd c')
  cmd('buffer ' .. fn.fnameescape(A))
  cmd('edit ' .. fn.fnameescape(B))
  cmd('buffer #')
end)

-- 7. Options and globals.  makeset's `setlocal` block under
--    'sessionoptions'+=localoptions/options, makemap's mappings, and
--    store_session_globals -- which keeps Strings and Numbers and nothing else.
S('options', function()
  cmd('edit ' .. fn.fnameescape(A))
  cmd('setlocal textwidth=37 tabstop=3 shiftwidth=3 expandtab spell spelllang=en_us')
  cmd('setlocal comments=b:%,:# commentstring=;;%s')
  cmd('split ' .. fn.fnameescape(B))
  cmd('setlocal filetype=lua nomodifiable readonly')
  cmd('lcd ' .. fn.fnameescape(WORK .. '/deep'))
  cmd('wincmd t')
  cmd('set wrapscan& ignorecase smartcase scrolloff=3 sidescrolloff=4')
  cmd('nnoremap <buffer> gQ :echo "buffer local"<CR>')
  cmd('inoremap <C-G><C-X> <Esc>')
  cmd('xnoremap <silent> gY "+y')
  -- var_flavour() calls a global a SESSION variable only when it starts with
  -- an uppercase letter AND carries a lowercase one somewhere after it:
  -- `SG_UPPER` is a *shada* variable and is not stored.  All six are here so
  -- that both arms of that test, and the three types that are skipped
  -- outright, are in the golden.
  cmd('let g:SgStr = "quote \\" back \\\\ tab \\t bar | end"')
  cmd("let g:SgNl = \"one\\ntwo\\rthree\"")
  cmd('let g:SgNum = -42')
  cmd('let g:SG_UPPER = 7')
  cmd('let g:sg_lower = 1')
  cmd('let g:SgFloat = 1.5')
  cmd('let g:SgList = [1, 2]')
  cmd('let g:SgDict = {"k": "v"}')
end)

-- 8. The awkward corners.  A blank window (the 'blank' flag), a one-line
--    window, a file name needing ses_escape_fname, an alternate file, and a
--    window whose buffer is `bufhidden=wipe`.
S('special', function()
  cmd('edit ' .. fn.fnameescape(B))
  cmd('edit ' .. fn.fnameescape(E))
  fn.cursor(2, 16)
  cmd('split')
  cmd('enew')
  cmd('split ' .. fn.fnameescape(A))
  cmd('resize 1')
  cmd('wincmd j')
  cmd('wincmd j')
  cmd('setlocal bufhidden=wipe')
  cmd('wincmd t')
end)

if SCENE == 'LIST' then
  for _, n in ipairs(ORDER) do out:write(n, '\n') end
  vim.cmd('qa!')
  return
end

--------------------------------------------------------------------------
-- Forms
--------------------------------------------------------------------------

local VDIR = WORK .. '/vdir'

local function reportfile(tag, path)
  local f = io.open(path, 'rb')
  if not f then
    say(tag, 'MISSING')
    return
  end
  local data = f:read('a')
  f:close()
  local nl = data:sub(-1) == '\n'
  if nl then data = data:sub(1, -2) end
  local n = 0
  for line in (data .. '\n'):gmatch('([^\n]*)\n') do
    n = n + 1
    say(tag, tostring(n), esc(scrub(line)))
  end
  say(tag, 'end', tostring(n), nl and 'eol' or 'NO-eol')
end

-- The forms.  Each is {name, options-to-set, the command, the file it writes}.
-- 'sessionoptions' and 'viewoptions' are restored after every form so that the
-- later :mkvimrc / :mkexrc records the same value in every scene.
local FORMS = {
  { 'sess-default', 'set sessionoptions&', 'mksession! %s', 's-default.vim' },
  {
    'sess-all',
    'set sessionoptions=blank,buffers,curdir,folds,globals,help,localoptions,options,resize,tabpages,terminal,winpos,winsize',
    'mksession! %s',
    's-all.vim',
  },
  { 'sess-sesdir', 'set sessionoptions=buffers,folds,sesdir,tabpages,winsize', 'mksession! %s', 's-sesdir.vim' },
  { 'sess-skiprtp', 'set sessionoptions=curdir,options,skiprtp,tabpages', 'mksession! %s', 's-skiprtp.vim' },
  { 'view-default', 'set viewoptions&', 'mkview! %s', 'v-default.vim' },
  { 'view-all', 'set viewoptions=cursor,curdir,folds,localoptions', 'mkview! %s', 'v-all.vim' },
  { 'mkvimrc', 'set sessionoptions& viewoptions&', 'mkvimrc! %s', 'r-vimrc.vim' },
  { 'mkexrc', 'set sessionoptions& viewoptions&', 'mkexrc! %s', 'r-exrc.vim' },
}

--------------------------------------------------------------------------
-- Run
--------------------------------------------------------------------------

local build = SCENES[SCENE]
if not build then
  say('scene', SCENE, 'UNKNOWN')
  vim.cmd('qa!')
  return
end

cmd('set nomore report=99999 shortmess+=F belloff=all')
cmd('set noswapfile nobackup noundofile hidden nofixeol')
cmd('set lines=24 columns=80')
cmd('set viewdir=' .. fn.fnameescape(VDIR))

local ok0, err0 = pcall(build)
say('scene', SCENE, ok0 and 'built' or ('BUILD-ERR ' .. esc(scrub(err0))))

-- The layout the forms are recorded against, so a scene that stopped building
-- differently is visible before the files are.
say('layout', SCENE, ('wins=%d tabs=%d cur=%s'):format(
  fn.winnr('$'), fn.tabpagenr('$'), table.concat(fn.getcurpos(), ',')))

for _, form in ipairs(FORMS) do
  local name, opts, tmpl, file = form[1], form[2], form[3], form[4]
  local path = WORK .. '/out/' .. SCENE .. '-' .. file
  pcall(cmd, opts)
  local ok, err = pcall(cmd, tmpl:format(fn.fnameescape(path)))
  if not ok then
    say('!', SCENE, name, esc(scrub(err)))
  end
  say('T', SCENE, name, 'this_session=' .. esc(scrub(vim.v.this_session)))
  reportfile('F\t' .. SCENE .. '\t' .. name, path)
  pcall(cmd, 'set sessionoptions& viewoptions&')
end

-- The numbered/default view files land in 'viewdir' under get_view_file()'s
-- encoding of the buffer's full path -- the encoding IS the answer here, so
-- the directory listing is reported, sorted, before the contents.
for _, arg in ipairs({ '', '3' }) do
  local ok, err = pcall(cmd, ('mkview! %s'):format(arg))
  if not ok then say('!', SCENE, 'view-vdir' .. arg, esc(scrub(err))) end
  -- The two 'viewdir' views MUST record different states, or `:loadview 3`
  -- and `:loadview` restore the same thing and nothing distinguishes them:
  -- a harness that ignored ex_loadview's argument entirely went NOT CAUGHT
  -- on the first mutation pass for exactly that reason.  Move the cursor and
  -- change the fold state between the two writes.
  pcall(cmd, 'normal! Gzz')
  pcall(cmd, 'setlocal foldlevel=0')
  pcall(cmd, 'normal! 0')
end
local okdir, names = pcall(fn.readdir, VDIR)
if not okdir or type(names) ~= 'table' then names = {} end
table.sort(names)
for i, n in ipairs(names) do
  say('V', SCENE, tostring(i), esc(scrub(n)))
  reportfile('F\t' .. SCENE .. '\tvdir-' .. tostring(i), VDIR .. '/' .. n)
end

-- ex_loadview: mangle the view state, load it back, and report what came back.
local function viewstate()
  local folds = {}
  for l = 1, math.min(fn.line('$'), 12) do
    folds[#folds + 1] = ('%d:%d/%d'):format(l, fn.foldlevel(l), fn.foldclosed(l))
  end
  return ('cur=%s fdl=%s fen=%s fdm=%s cwd=%s | %s'):format(
    table.concat(fn.getcurpos(), ','), vim.o.foldlevel, tostring(vim.o.foldenable),
    vim.o.foldmethod, scrub(fn.getcwd()), table.concat(folds, ' '))
end

say('L', SCENE, 'before', esc(scrub(viewstate())))
pcall(cmd, 'normal! ggzR')
pcall(cmd, 'setlocal foldmethod=manual foldlevel=99')
pcall(cmd, 'normal! zE')
say('L', SCENE, 'mangled', esc(scrub(viewstate())))
for _, arg in ipairs({ '3', '' }) do
  local ok, err = pcall(cmd, ('loadview %s'):format(arg))
  say('L', SCENE, 'loadview[' .. arg .. ']',
    ok and esc(scrub(viewstate())) or ('E ' .. esc(scrub(err))))
end

-- The error arms of ex_mkrc: no `!` over an existing file, a directory as the
-- target, and an unwritable 'viewdir'.
local exists = WORK .. '/out/' .. SCENE .. '-s-default.vim'
for _, probe in ipairs({
  { 'exists-nobang', 'mksession ' .. fn.fnameescape(exists) },
  { 'target-isdir', 'mksession! ' .. fn.fnameescape(WORK .. '/deep') },
  { 'view-nofile', 'mkview! ' .. fn.fnameescape(WORK .. '/deep/nope/x.vim') },
}) do
  local ok, err = pcall(cmd, probe[2])
  say('X', SCENE, probe[1], ok and 'ok' or esc(scrub(err)))
end

vim.cmd('qa!')
