-- Differential probe for Ex command *execution* (ex_docmd.rs).
--
-- The parse probe covers what `nvim_parse_cmd` decided; this one covers what
-- `do_cmdline`/`do_one_cmd` then did with it -- the range actually applied,
-- the modifier actually in force while the command ran, the register the
-- count landed in, the message, and the re-entrant forms (`:execute`,
-- `:global`, `:normal`, sourced lines) that the parser never sees.
--
-- A case is (scene, command line); the recording is the whole observable
-- state afterwards, including the exact message text.
--
-- Restartable in expr-run.sh's sense.
--
--   ex-run.sh <nvim> <runtime> <outdir> <label> excmd
--   PROBE_LIST=1 ... > cases.txt
--
-- Rehomed from the phase-14 scratchpad at B17-5.  Two environment leaks were
-- scrubbed on the way (`pwd`, `:checkhealth`); see EX_WORK below and the case
-- list's `checkhealth` entry.

local out = io.stdout
out:setvbuf('line')

local FROM = tonumber(os.getenv('PROBE_FROM') or '1')
local LIST = os.getenv('PROBE_LIST')

local api = vim.api
local function ev(e) return api.nvim_eval(e) end

-- The runner cds into EX_WORK and points $VIMRUNTIME at the tree under test,
-- so both are known strings rather than whatever the caller's shell had.
-- `:pwd` prints the first and half of nvim's own messages name the second.
local WORK = os.getenv('EX_WORK') or ''
local RT = os.getenv('EX_RUNTIME') or (os.getenv('VIMRUNTIME') or '')
local HOME = os.getenv('HOME') or ''

local function lit(s) return (s:gsub('[%^%$%(%)%%%.%[%]%*%+%-%?]', '%%%1')) end

local function scrub(s)
  s = tostring(s)
  s = s:gsub('0x%x%x%x%x%x%x%x%x+', '0xADDR')
  s = s:gsub('Error in pre%-vimrc command line:\n', '')
  -- `:checkhealth` reports the log file's size, which grows as the probe
  -- itself runs and differs between two worktrees.
  s = s:gsub('Log size: %d+ KB', 'Log size: N KB')
  -- The three paths any message can name.  Longest first: $VIMRUNTIME and
  -- $HOME are usually prefixes of nothing, but the work directory is a
  -- temporary whose NAME LENGTH would otherwise reach the artifact through
  -- every elided message.
  if WORK ~= '' then s = s:gsub(lit(WORK), '<WORK>') end
  if RT ~= '' then s = s:gsub(lit(RT), '<RT>') end
  if HOME ~= '' and HOME ~= '/' then s = s:gsub(lit(HOME), '<HOME>') end
  s = s:gsub('/nix/store/%w+%-', '<STORE>/')
  s = s:gsub('\n', '\\n'):gsub('\r', '\\r'):gsub('\t', '\\t')
  return s
end

--------------------------------------------------------------------------
-- Scenes
--------------------------------------------------------------------------

local SCENES = {
  text = {
    lines = {
      'The quick brown fox jumps over the lazy dog.',
      'second line of the scene buffer',
      'third line with brown in it too',
      'fourth line',
      'fifth line, the cursor starts here',
      'sixth line',
      'seventh line',
      'eighth and last line',
    },
    row = 5, col = 3,
  },
  short = {
    lines = { 'only line' },
    row = 1, col = 1,
  },
  folds = {
    lines = {
      'level0 a', '  level1 b', '    level2 c', '    level2 d',
      '  level1 e', 'level0 f', '  level1 g', '    level2 h', 'level0 i',
    },
    row = 3, col = 1,
    opts = { 'setlocal foldmethod=indent shiftwidth=2 expandtab' },
  },
}

--------------------------------------------------------------------------
-- Case list
--------------------------------------------------------------------------

local cases = {}
local seen = {}
local function C(scene, line)
  local k = scene .. '\1' .. line
  if seen[k] then return end
  seen[k] = true
  cases[#cases + 1] = { scene, line }
end
local function T(line) C('text', line) end

-- 1. Ranges applied to commands that show what they got.
local ADDRS = {
  '', '.', '$', '%', '0', '1', '3', '8', '9', '99',
  '2147483647', '9223372036854775807', '99999999999999999999',
  "'a", "'z", "'<,'>", "'q", '/brown/', '?brown?', '\\/', '\\?',
  '+2', '-2', '.+2', '.-2', '$-1', '1,3', '3,1', '$,1', '1;3', '3;1',
  '.,$', '1,$', '0,0', '0,1', "'a,'z", '/brown/,/line/', '.,.+3', '.-1,.+1',
}
for _, a in ipairs(ADDRS) do
  for _, c in ipairs({ 'print', 'number', 'list', 'delete', 'yank', 'copy 0',
                       'move 0', 'join', 'normal! x', '>', '<', 's/o/0/',
                       'g/e/print', 'sort', 'left', 'right 5', 'center' }) do
    T(a .. c)
  end
end

-- 2. Modifiers actually in force.
local MODS = { 'silent', 'silent!', 'unsilent', 'verbose', '3verbose',
               'keeppatterns', 'keepjumps', 'keepmarks', 'keepalt',
               'lockmarks', 'noautocmd', 'noswapfile', 'sandbox',
               'confirm', 'browse', 'hide', 'legacy' }
for _, m in ipairs(MODS) do
  for _, c in ipairs({ 'print', 'echo "x"', 'echoerr "x"', '1,3delete',
                       's/o/0/', 'normal! x', 'let g:probe = 1',
                       'call setline(1, "changed")', '3,4yank' }) do
    T(m .. ' ' .. c)
  end
end
for _, m in ipairs({ 'silent', 'lockmarks', 'keepjumps', 'noautocmd' }) do
  T(m .. ' ' .. m .. ' print')
  T('silent ' .. m .. ' keeppatterns 1,3print')
end
-- window-splitting modifiers (they change the window layout, which readback sees)
for _, m in ipairs({ 'vertical', 'horizontal', 'aboveleft', 'belowright',
                     'topleft', 'botright', 'leftabove', 'rightbelow', 'tab', '2tab' }) do
  T(m .. ' new')
  T(m .. ' split')
end
-- :filter over a listing command
for _, f in ipairs({ 'filter /line/ ', 'filter! /line/ ', 'filter line ' }) do
  T(f .. 'ls')
  T(f .. 'set')
end

-- 3. Bang, register, count on the commands that take them.
for _, r in ipairs({ 'a', 'z', 'A', '"', '0', '1', '-', '_', '/', ':', '.', '%', '#', '=', '+', 'Q' }) do
  T('1,2delete ' .. r)
  T('1,2yank ' .. r)
  T('put ' .. r)
  T('put! ' .. r)
  T('1,2delete ' .. r .. ' 3')
  T('yank ' .. r .. ' 2')
end
for _, n in ipairs({ '', '0', '1', '2', '5', '99', '2147483647', '2147483648',
                     '9223372036854775807', '99999999999999999999' }) do
  T('delete ' .. n)
  T('2yank ' .. n)
  T('normal! ' .. n .. 'x')
  T('>' .. ' ' .. n)
  T('<' .. ' ' .. n)
  T('3,4join ' .. n)
  T('earlier ' .. n)
  T('later ' .. n)
end
-- `:sleep` really does sleep, so only the small and the rejected forms
-- belong here; the large counts are covered by the parse probe.
for _, l in ipairs({ 'sleep 0m', 'sleep 1m', 'sleep 0', 'sleep', 'sleep x',
                     'sleep 5x', 'sleep 1m!', '3sleep m' }) do T(l) end
for _, c in ipairs({ 'undo', 'redo', 'nohlsearch', 'file x', 'set',
                     'redraw', 'redrawstatus', 'redrawtabline', 'setfiletype c',
                     'startinsert', 'stopinsert', 'digraphs', 'pwd', 'cd .',
                     -- A bare `:checkhealth` runs vim.provider and
                     -- vim.treesitter as well, and their report names the
                     -- nvim version, the build type, the ripgrep/git/curl
                     -- versions found on $PATH, the LSP log's size in KB and
                     -- seven /nix/store paths: a case that re-baselines
                     -- itself on any machine, on any day, and against any
                     -- binary built at another revision.  `vim.deprecated`
                     -- is four lines and none of them are environment.
                     'checkpath', 'checkhealth vim.deprecated',
                     'runtime x', 'behave xterm' }) do
  T(c); T(c:gsub('^(%a+)', '%1!'))
end

-- 4. Bar separation, comments, trailing garbage, ranges without a command.
for _, l in ipairs({
  'print | print', 'echo 1 | echo 2', 'normal! ix|y', 'normal ix|y',
  '1,2print | 3,4print', 's/o/0/ | print', 'g/e/print | print',
  'let g:a = 1 | let g:b = 2', 'echo "a|b"', 'print "comment',
  '" whole comment', 'print xyz', 'undo 5 xyz', '1,2', '5', '$', '0',
  'execute "print"', 'execute "1,2print"', 'execute "silent print"',
  'execute "echo" "1"', 'execute', 'execute ""', 'execute "execute \\"print\\""',
  'call execute("print")', 'silent execute "echoerr \'x\'"',
}) do T(l) end

-- 5. :normal re-entering the normal-mode state machine.
for _, l in ipairs({
  'normal! ggdG', 'normal! x', 'normal x', 'normal! 3x', 'normal! dd',
  '1,3normal! x', '%normal! A;', 'normal! ihello\27', 'normal ihello\27',
  'normal!', 'normal', 'normal! v$y', 'normal! qaxxq@a',
  'normal! 2147483647x', 'normal! 9223372036854775807x',
  'silent normal! x', 'keepjumps normal! G', 'lockmarks normal! majj',
  'normal! :print\r', 'execute "normal! \\<C-V>jjIX\\<Esc>"',
  'g/e/normal! x', '1,3g/e/normal! A!',
}) do T(l) end

-- 6. :global and :vglobal, including nesting and the empty pattern.
for _, l in ipairs({
  'g/e/print', 'g/e/delete', 'g/e/s/e/E/', 'v/e/print', 'v/e/delete',
  'g!/e/print', 'global/e/print', 'global! /e/ print', 'g/nomatch/print',
  'g//print', 'g/e/', 'g/e/normal! x', 'g/e/g/o/print',
  '1,3g/e/print', 'g/e/1,2print', 'g/e/undo', 'g/e/execute "print"',
  'g/^/move 0', 'g/e/join', 'g/e/t.',
}) do T(l) end

-- 7. Errors: the exact E-number and text.
for _, l in ipairs({
  'zzz', 'abcdefgh', 'delete zz', 'yank !!', 'put ~~', 'normal!!',
  'k', 'mark', 'mark toolong', 'k ab', "'x", "'xp", '/nomatch/p', '?nomatch?p',
  '5,1p', '$,1p', '1,2,3,4p', '99999p', '0delete', '0yank', '0print',
  'redir @', 'redir @a>', 'redir END', 'sleep x', 'sleep 5x',
  'wincmd', 'wincmd zz', 'argdelete', 'buffer 999', 'tabclose 99',
  'edit ++bogus f', 'edit ++', 'set nosuchoption', 'unlet g:nosuchvar',
  'call nosuchfunc()', 'echo nosuchvar', 'if 1', 'endif', 'else', 'endwhile',
  'endfor', 'endtry', 'catch', 'finally', 'continue', 'break', 'return',
  'throw "x"', '<<<<<<<<', '>>>>>>>>', '.!', '1,2!', '&&&', '~~~',
  'substitute', 's/', 's/a', 's/a/', 's//x/', 'sort!!', '@', '@@',
}) do T(l) end

-- 8. Command-name resolution and the one-letter forms that execute.
for _, l in ipairs({
  'p', 'pr', 'print', 'nu', 'number', 'l', 'list',
  'd', 'de', 'del', 'delete', 'dl', 'dp', 'delp', 'y', 'ya', 'yank',
  'k a', 'ka', 'kb', 'ma a', 'mark a', 't0', 'co0', 'copy 0', 'm0', 'move 0',
  '2t.', '2co.', 's/o/0/', '&', '&&', '~', '=', '.=', '1,3=',
  'j', 'join', 'j!', '>', '>>', '>>>', '<', '<<', 'le', 'left', 'ri', 'right',
  'ce', 'center', 'sor', 'sort', 'sort!', 'sort u', 'sort n', 'sort i',
  'X', 'Next', 'Print', 'z', 'z=', 'z+', 'z-', 'z.',
}) do T(l) end

-- 9. Buffer/window/tab commands with counts (the non-line address kinds).
for _, l in ipairs({
  'new', 'split', 'vsplit', 'close', 'only', 'hide', 'wincmd w', 'wincmd p',
  '2wincmd w', 'wincmd o', 'resize 5', 'vertical resize 20', 'winsize 80 24',
  'tabnew', 'tabnext', 'tabprevious', 'tabclose', 'tabonly', 'tabs',
  '2tabnext', 'tabmove 0', 'tabmove +1', 'enew', 'bnext', 'bprevious',
  'blast', 'bfirst', 'bmodified', 'ls', 'buffers', 'files',
  'argument', 'next', 'previous', 'first', 'last', 'args',
}) do T(l) end

-- 10. The short/edge scenes.
for _, l in ipairs({
  '%print', '%delete', '$print', '1,$print', '0put', 'put', 'join',
  '.,$join', 'normal! dd', 'g/./delete', 'v/./delete', 'undo', 'redo',
}) do C('short', l) end
for _, l in ipairs({
  'normal! zM', 'normal! zR', '%foldopen', '%foldclose', '3,7foldopen!',
  'fold', '1,4fold', 'foldopen', 'foldclose', 'folddoopen print',
  'folddoclosed print', 'normal! zj', 'normal! zk',
}) do C('folds', l) end

-- 11. `nvim_cmd` builds an ExArg from a Dict, so it reaches the range and
--     count plumbing with values no command line can spell. The count
--     extremes here abort a pre-slice binary in `set_cmd_count`.
local NVIM_CMD = {
  '{cmd="print"}', '{cmd="print", count=0}', '{cmd="print", count=1}',
  '{cmd="print", count=2}', '{cmd="print", count=2147483647}',
  '{cmd="print", count=-1}', '{cmd="print", count=-2147483648}',
  '{cmd="delete", count=0}', '{cmd="delete", count=3}',
  '{cmd="normal", args={"x"}, count=0}', '{cmd="normal", args={"x"}, count=3}',
  '{cmd="buffer", count=0}', '{cmd="buffer", count=1}',
  '{cmd="wincmd", args={"w"}, count=0}', '{cmd="tabnext", count=0}',
  '{cmd="print", range={1}}', '{cmd="print", range={1,3}}',
  '{cmd="print", range={3,1}}', '{cmd="print", range={0}}',
  '{cmd="print", range={2147483647}}', '{cmd="print", range={1,2147483647}}',
  '{cmd="yank", reg="a"}', '{cmd="yank", reg="a", count=2}',
  '{cmd="write", bang=true}', '{cmd="print", mods={silent=true}}',
  '{cmd="print", mods={verbose=3}}', '{cmd="print", mods={tab=1}}',
  '{cmd="new", mods={vertical=true, split="topleft"}}',
  '{cmd="print", mods={emsg_silent=true, keeppatterns=true}}',
  '{cmd="nosuchcommand"}', '{cmd=""}', '{cmd="print", nargs="0"}',
  '{cmd="substitute", args={"/o/0/"}}',
  '{cmd="print", magic={file=true, bar=false}}',
}
for _, d in ipairs(NVIM_CMD) do
  T('lua pcall(vim.api.nvim_cmd, ' .. d .. ', {})')
  T('lua print(vim.inspect(select(2, pcall(vim.api.nvim_cmd, ' .. d .. ', {output=true}))))')
end

if LIST then
  for i, c in ipairs(cases) do
    out:write(('%d\t%s\t%s\n'):format(i, c[1], scrub(c[2])))
  end
  os.exit(0)
end

--------------------------------------------------------------------------
-- Running a case
--------------------------------------------------------------------------

local REGS = { '"', '0', '1', 'a', 'z', '-' }
local MARKS = { 'a', 'z', '<', '>', '[', ']', '.', "'" }

api.nvim_command('set nomore report=99999 shortmess+=F belloff=all undolevels=1000')
api.nvim_command('set noswapfile nobackup noundofile hidden nofoldenable')
api.nvim_command('set laststatus=0 nonumber')

local function setup(name)
  local sc = SCENES[name]
  pcall(api.nvim_command, 'normal! \27\27')
  pcall(api.nvim_command, 'silent! tabonly!')
  pcall(api.nvim_command, 'silent! only!')
  local prev = api.nvim_get_current_buf()
  pcall(api.nvim_command, 'enew!')
  pcall(api.nvim_command, 'bwipeout! ' .. prev)
  api.nvim_command('setlocal buftype= modifiable')
  api.nvim_command('set foldmethod=manual shiftwidth=8 noexpandtab nofoldenable')
  api.nvim_command('set nowrap nostartofline whichwrap= selection=inclusive virtualedit=')
  api.nvim_command('set report=99999 nohlsearch')
  for _, r in ipairs(REGS) do pcall(vim.fn.setreg, r, '') end
  api.nvim_buf_set_lines(0, 0, -1, false, sc.lines)
  for _, c in ipairs(sc.opts or {}) do api.nvim_command(c) end
  pcall(api.nvim_command, 'delmarks!')
  pcall(vim.fn.cursor, sc.row, sc.col)
  pcall(api.nvim_command, 'normal! majjmz')
  pcall(vim.fn.cursor, sc.row, sc.col)
  pcall(api.nvim_command, 'normal! vjl\27')
  pcall(vim.fn.cursor, sc.row, sc.col)
  pcall(api.nvim_command, 'let @/ = "brown"')
  pcall(api.nvim_command, 'unlet! g:probe')
  api.nvim_command('let &l:modified = 0')
  api.nvim_command('silent! clearjumps')
  api.nvim_command('messages clear')
  api.nvim_command('let v:errmsg = ""')
end

local function readback()
  local f = {}
  local function add(x) f[#f + 1] = scrub(x) end
  local ok, p = pcall(vim.fn.getcurpos)
  add(ok and table.concat(p, ',') or 'ERR')
  for _, m in ipairs(MARKS) do
    local ok2, v = pcall(vim.fn.getpos, "'" .. m)
    add(m .. '=' .. (ok2 and table.concat(v, ',') or 'ERR'))
  end
  local ok3, lines = pcall(api.nvim_buf_get_lines, 0, 0, -1, false)
  add('lines=' .. (ok3 and table.concat(lines, '|') or 'ERR'))
  add('mod=' .. tostring(ev('&modified')))
  for _, r in ipairs(REGS) do
    local o1, v = pcall(vim.fn.getreg, r)
    local o2, t = pcall(vim.fn.getregtype, r)
    add(('r%s=%s/%s'):format(r, o1 and v or 'ERR', o2 and t or 'ERR'))
  end
  add('search=' .. tostring(vim.fn.getreg('/')))
  add('wins=' .. tostring(vim.fn.winnr('$')) .. '/' .. tostring(vim.fn.tabpagenr('$'))
      .. '/' .. tostring(#vim.fn.getbufinfo()))
  local jl = vim.fn.getjumplist()
  add(('jump=%d/%d'):format(#jl[1], jl[2]))
  local cl = vim.fn.getchangelist()
  add(('chg=%d/%d'):format(#cl[1], cl[2] or -1))
  add('alt=' .. tostring(vim.fn.bufname('#')))
  add('probe=' .. tostring(ev('exists("g:probe") ? g:probe : "-"')))
  add('errmsg=' .. tostring(ev('v:errmsg')))
  local o, msgs = pcall(api.nvim_exec2, 'messages', { output = true })
  add('msg=' .. (o and (msgs.output or '') or 'ERR'))
  return table.concat(f, ' | ')
end

for i = FROM, #cases do
  local scene, line = cases[i][1], cases[i][2]
  local ok0 = pcall(setup, scene)
  local ok, err = pcall(api.nvim_command, line)
  local ok2, r = pcall(readback)
  -- `--headless` writes :print/:list/:ls output straight to stdout, so a
  -- record has to announce itself: the runner and the diff both key on the
  -- CASE marker rather than on line position.
  out:write(('\nCASE\t%d\t%s\t%s\t%s\t%s\n'):format(
    i, scene, scrub(line), ok and 'ok' or ('E ' .. scrub(err)),
    ok2 and r or ('READBACK-ERR ' .. scrub(r))))
  if not ok0 then out:write(('CASE\t%d\tSETUP-FAILED\n'):format(i)) end
end

vim.cmd('qa!')
