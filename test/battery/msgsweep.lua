-- msgsweep -- the message-pipeline differential.  Driven by msgsweep.sh,
-- which builds the sandbox; read that header first.
--
-- The subsystem is `message/` and everything that hands it text: `:echo`,
-- `:echon`, `:echomsg`, `:echoerr`, `:echohl`, the `emsg` family and the
-- `shortmess` flags that shape it, `msg_trunc`/`msg_may_trunc` at several
-- widths, the message history (`:messages`, `:messages clear`, a count,
-- 'msghistory'), `v:errmsg`/`v:statusmsg`/`v:warningmsg`, `:redir` and
-- `execute()` capture, `nvim_echo` chunks, `confirm()`, the display of
-- every value type, and control/multibyte text through `msg_outtrans` and
-- `str2special`.
--
-- TWO KINDS OF ANSWER, like scrsweep (whose harness this reuses: the
-- functional suite's `testnvim` + `ui/screen`, a real UI client):
--
--   .vals   a value per case -- the captured text of a command
--           (`nvim_exec2(.., {output = true})`, which is the same capture
--           `execute()` uses), an expression's answer, or an error's text.
--   .txt    the GRID: what a message looks like when it is drawn.  Width,
--           truncation, the hit-enter and more prompts and 'cmdheight'
--           only exist on a screen, so those cases type their command with
--           `feed` and snapshot the grid.  `.attrs` is the attribute table
--           the grid's ids refer to (scrsweep's reason: ids are assigned in
--           discovery order, so an id alone is not an oracle).
--
-- Some cases run under `ext_messages`, where a message arrives as
-- `msg_show` chunks carrying a kind and a highlight: that is how
-- `nvim_echo`'s chunks and the kind of every emsg path are seen.
--
-- EVERY RECORD CARRIES ITS CASE NUMBER and the run ends with
-- `## LAST <n>`.  A driver that silently stops early (p34's `expr` row
-- lost half its cases to a `return` for four slices) reads IDENTICAL
-- against a base that stopped at the same place; msgverify.sh checks the
-- `## LAST` line against the case count this file declares.

local root = assert(os.getenv('NVIMRS_ROOT'), 'NVIMRS_ROOT unset')
package.path = table.concat({
  root .. '/?.lua',
  root .. '/?/init.lua',
  root .. '/test/?.lua',
  root .. '/test/?/init.lua',
  root .. '/runtime/lua/?.lua',
}, ';') .. ';' .. package.path

local outdir, label = _G.arg[1], _G.arg[2]
local work = assert(os.getenv('MSGSWEEP_WORK'), 'MSGSWEEP_WORK unset')

local n = require('test.functional.testnvim')()
local Screen = require('test.functional.ui.screen')

local txt, attrs, vals = {}, {}, {}
local function say(t, ...)
  t[#t + 1] = table.concat({ ... }, '')
end

local trace = (os.getenv('MSGSWEEP_TRACE') or '') ~= ''
local case_no = 0
local function case(name)
  case_no = case_no + 1
  if trace then
    io.stderr:write(('msgsweep: %04d %s\n'):format(case_no, name))
  end
  return ('%04d %s'):format(case_no, name)
end

local function ins(v)
  return vim.inspect(v, { newline = ' ', indent = '' })
end

--- Escape to one printable line: control bytes and high bytes as `\xNN`.
local function esc(s)
  s = tostring(s):gsub(vim.pesc(work), '<WORK>')
  return (s:gsub('[%c\128-\255\\]', function(c)
    return ('\\x%02x'):format(c:byte())
  end))
end

--- A Lua-side error carries a traceback whose frames are ASLR addresses.
local function errtext(e)
  local s = tostring(e)
  s = s:gsub('%s*stack traceback:.*$', '')
  s = s:gsub('0x%x+', '0xADDR')
  -- The harness prefixes its own source position (an elided absolute path
  -- and a line number in someone else's file).
  s = s:gsub('^[^\n]-%.lua:%d+: ', '')
  return s
end

local function attr_repr(a)
  if a == nil then
    return 'nil'
  end
  local keys = {}
  for k in pairs(a) do
    keys[#keys + 1] = k
  end
  table.sort(keys)
  local parts = {}
  for _, k in ipairs(keys) do
    local v = a[k]
    if type(v) == 'table' then
      v = attr_repr(v)
    end
    parts[#parts + 1] = ('%s=%s'):format(k, tostring(v))
  end
  return '{' .. table.concat(parts, ' ') .. '}'
end

-- ----------------------------------------------------------- recorders

--- Run an Ex command line and record its captured output, or its error.
local function cap(name, cmd)
  local tag = case(name)
  local ok, res = pcall(n.api.nvim_exec2, cmd, { output = true })
  if ok then
    say(vals, tag, ' O ', esc(res.output))
  else
    say(vals, tag, ' ! ', esc(errtext(res)))
  end
end

--- Record an expression's value.
local function ev(name, expr)
  local tag = case(name)
  local ok, res = pcall(n.api.nvim_eval, expr)
  if ok then
    say(vals, tag, ' = ', esc(ins(res)))
  else
    say(vals, tag, ' ! ', esc(errtext(res)))
  end
end

--- Run a command whose output is not the measurement; failures still show.
local function run(cmd)
  local ok, res = pcall(n.api.nvim_exec2, cmd, { output = false })
  if not ok then
    say(vals, '     setup ', esc(cmd), ' ! ', esc(errtext(res)))
  end
end

--- The three message variables, in one row.
local function msgvars(name)
  ev(name, '[v:errmsg, v:statusmsg, v:warningmsg]')
end

--- The screen of the current child; `start` sets it.
local screen

--- Wait for the child to go quiet: three samples of (mode, blocking,
--- grid) apart, the last two equal.  `nvim_get_mode` is a FAST call, so it
--- is answered even at a hit-enter, more or dialog prompt -- where any
--- other request (`poke_eventloop`'s `nvim_eval` included) WAITS for a key
--- and would deadlock this driver.  The answer is the last mode sampled.
local function quiesce()
  local prev, mode
  for i = 1, 200 do
    screen:sleep(10)
    mode = n.api.nvim_get_mode()
    local key = ('%s %s %s'):format(mode.mode, tostring(mode.blocking), screen:get_snapshot().grid)
    if key == prev and i >= 3 then
      break
    end
    prev = key
  end
  return mode
end

--- Snapshot the grid (and any ext_ state) once the child is quiet.
local function snap(name)
  local tag = case(name)
  local mode = quiesce()
  local kwargs, ext = screen:get_snapshot()
  say(txt, '--- ', tag)
  for _, line in ipairs(vim.split(kwargs.grid, '\n')) do
    say(txt, esc(line))
  end
  local ids = kwargs.attr_ids or {}
  local keys = {}
  for k in pairs(ids) do
    keys[#keys + 1] = k
  end
  table.sort(keys, function(a, b)
    return tostring(a) < tostring(b)
  end)
  say(attrs, '--- ', tag)
  for _, k in ipairs(keys) do
    say(attrs, ('  [%s] %s'):format(tostring(k), attr_repr(ids[k])))
  end
  for _, k in ipairs({ 'cmdline', 'messages', 'msg_history', 'showmode', 'ruler' }) do
    if ext[k] ~= nil and next(ext[k]) ~= nil then
      say(txt, ('  ext.%s = %s'):format(k, esc(ins(ext[k]))))
    end
  end
  say(txt, ('  mode=%s blocking=%s'):format(mode.mode, tostring(mode.blocking)))
end

--- Type keys, then wait until the child has acted on them.
local function feed(keys)
  n.feed(keys)
  quiesce()
end

--- Leave any prompt: hit-enter, more, a dialog.  `<C-c>` would set
--- got_int and leak into the next case; `<Esc>` is what every prompt here
--- accepts, and a second one after the first lands in Normal mode.
local function settle()
  for _ = 1, 5 do
    local m = quiesce()
    if m.mode == 'n' and not m.blocking then
      return
    end
    n.feed('<Esc>')
  end
end

--- A fresh child with a deterministic screen.
local function start(w, h, opts)
  n.clear()
  screen = Screen.new(w or 40, h or 8, opts)
  n.command('set noruler noshowcmd laststatus=1 belloff=all')
  n.command('cd ' .. work)
end

--- A fresh child with no UI: the value-only sections.
local function plain()
  n.clear()
  n.command('set noruler noshowcmd belloff=all nomore')
  n.command('cd ' .. work)
end

local S = {}
local function section(name, fn)
  S[#S + 1] = { name = name, fn = fn }
end

-- ================================================================ values

-- `:echo` of every value type, through the capture.  `echo_string` is
-- the formatter (`encode_tv2echo`), float formatting is `%g`-ish with the
-- `.0` suffix, and a string goes through `msg_outtrans_len` (control bytes
-- become `^X`, a byte >= 0x80 that is not UTF-8 becomes `<xx>`).
section('s1-echo-types', function()
  plain()
  local exprs = {
    '0', '-1', '9223372036854775807', '-9223372036854775807 - 1', '0x10', '0b101', '0o17',
    '1.0', '-0.0', '0.1 + 0.2', '1.5e10', '1.0e100', '1.0e-5', '123456789.0', '1.0 / 0', '-1.0 / 0',
    '0.0 / 0', '3.14159265358979', '1.0e15', '1.0e16', '100000.0', '0.0001',
    '""', '"abc"', '"a\\tb"', '"a\\nb"', '"\\x01\\x7f"', '"\\x80\\xff"', '"\\u00e9\\u65e5\\u672c"',
    '"\\u200b|\\u0301|"', '"\\e[31m"', '"x\\<C-a>y"', 'repeat("w", 200)',
    '[]', '{}', '[1, "two", 3.0]', '[[], [[]], {}]', "{'a': {'b': [1, {}]}}", "{'k': 'v', 'n': 1}",
    '[v:true, v:false, v:null]', 'v:true', 'v:null', 'v:none', '0z', '0zDEADBEEF', '0z00.11.22',
    "function('tr')", "function('tr', [1])", "function('tr', [1], {'a': 1})", '{x -> x + 1}',
    "funcref('strlen')", 'v:_null_list', 'v:_null_dict', 'v:_null_string', 'v:_null_blob',
    '[v:_null_list, v:_null_dict, v:_null_string, v:_null_blob]',
    '[1, [2, [3, [4, [5]]]]]', "{'nested': {'list': [0z01, function('tr')]}}",
  }
  for _, e in ipairs(exprs) do
    cap('echo ' .. e, 'echo ' .. e)
  end
  for _, e in ipairs({ '"a" "b"', '1 2 3', '[1] {}', '"x" 1.5 0z01' }) do
    cap('echo-many ' .. e, 'echo ' .. e)
    cap('echon-many ' .. e, 'echon ' .. e)
  end
  -- A recursive container prints its back-reference as `[...]` / `{...}`.
  run('let g:l = [1] | call add(g:l, g:l) | let g:d = {} | let g:d.d = g:d')
  cap('echo recursive list', 'echo g:l')
  cap('echo recursive dict', 'echo g:d')
  cap('echo string() recursive', 'echo string(g:l)')
  cap('echo mixed recursive', 'echo [g:l, g:d]')
  -- `string()` and `:echo` disagree on strings and floats inside lists.
  for _, e in ipairs({ '"q"', '["q", 1.0]', "{'s': \"it's\"}", '1.0e100', '[1.0 / 0]' }) do
    cap('echo string ' .. e, 'echo string(' .. e .. ')')
  end
  cap('echo float-format', 'echo printf("%g %e %f %.3f %10.2f", 1.5, 1.5, 1.5, 1.5, 1.5)')
  cap('echon then echo', 'echon "a" | echon "b" | echo "c" | echon "d"')
  cap('echo empty line', 'echo')
  cap('echo multiline value', 'echo "one\\ntwo\\nthree"')
  cap('echo crlf', 'echo "a\\r\\nb"')
  cap('echo nul in blob', 'echo 0z00')
end)

-- The error channel: E-codes from the common paths, `:echoerr`, the
-- "Error detected while processing" header, try/catch, `:silent!` and the
-- message variables after each.
section('s2-emsg', function()
  plain()
  local cmds = {
    'nosuchcommand', 'echo nosuchvar', 'call NoSuchFunc()', 'let x =', 'echo [1][5]', "echo {}['k']",
    'echo 1 +', 'echo "unterminated', 'let [a, b] = [1]', 'echo 1 2 +', 'throw "custom"',
    'throw 42', 'echoerr "plain echoerr"', 'echoerr "a" "b" 3', 'echoerr [1, 2]',
    'set nosuchoption', 'set tabstop=x', 'buffer 999', 'normal! :nosuch\r', 'call assert_true(0)',
    'edit nonexist/dir/file.txt | write', '1,2,3d', 'echo printf("%d", "x")', 'unlet g:nosuch',
    'let g:x = 1 | lockvar g:x | let g:x = 2', 'call feedkeys()', 'echo strlen()', 'echo strlen(1, 2)',
  }
  -- Each command three ways: through the API (the error comes back as
  -- the API's text), under `silent!` (the error sets `v:errmsg` and
  -- nothing else), and inside `:try` (the error is `v:exception`).
  for _, c in ipairs(cmds) do
    cap('err ' .. c, c)
    run('silent! ' .. c)
    msgvars('vars after silent! ' .. c)
    cap('try ' .. c, 'try | exe ' .. "'" .. c:gsub("'", "''") .. "'" .. ' | catch | echo v:exception | endtry')
    run('let v:errmsg = "" | let v:statusmsg = "" | let v:warningmsg = "" | silent! unlockvar g:x')
  end
  -- Errors inside a function carry the header and the line number.
  run(table.concat({
    'function! Fails()',
    '  echo "before"',
    '  call nosuch()',
    '  echo "after"',
    'endfunction',
    'function! FailsAbort() abort',
    '  echo "before"',
    '  call nosuch()',
    '  echo "after"',
    'endfunction',
    'function! Nested()',
    '  call Fails()',
    'endfunction',
  }, '\n'))
  cap('func error', 'call Fails()')
  cap('func error abort', 'call FailsAbort()')
  cap('func error nested', 'call Nested()')
  cap('try catch', 'try | call nosuch() | catch | echo v:exception "|" v:throwpoint | endtry')
  cap('try finally', 'try | echo "t" | finally | echo "f" | endtry')
  cap('try echoerr', 'try | echoerr "boom" | catch | echo v:exception | endtry')
  cap('silent! error', 'silent! call nosuch() | echo "after:" v:errmsg')
  cap('silent error', 'silent call nosuch()')
  cap('silent echo', 'silent echo "hidden"')
  cap('silent echomsg', 'silent echomsg "hidden-hist"')
  cap('unsilent', 'silent unsilent echo "shown"')
  cap('emsg_silent in try', 'try | silent! throw "x" | catch | echo "caught" v:exception | endtry')
  msgvars('vars end s2')
  -- `shortmess` flags against the message text.
  local file = 'msgfile.txt'
  run('call writefile(["one", "two", "three"], "' .. file .. '")')
  for _, sm in ipairs({ '', 'a', 'l', 'm', 'r', 'w', 'x', 'n', 'F', 'W', 's', 'S', 'T', 't', 'I', 'aoOstTWF' }) do
    run('set shortmess=' .. sm)
    cap('sm=' .. sm .. ' edit', 'edit! ' .. file)
    cap('sm=' .. sm .. ' write', 'write!')
    cap('sm=' .. sm .. ' file', 'file')
    run('call setline(1, ["a", "b", "a"]) | 3 | let @/ = "a"')
    cap('sm=' .. sm .. ' search wrap n', 'normal! n')
    cap('sm=' .. sm .. ' search wrap N', 'normal! N')
    cap('sm=' .. sm .. ' search count', 'set shortmess-=S | normal! n')
    cap('sm=' .. sm .. ' search fail', 'silent! /zzz/')
    msgvars('sm=' .. sm .. ' vars')
    run('let v:errmsg = "" | let v:statusmsg = "" | let v:warningmsg = "" | silent! bwipe!')
  end
  run('set shortmess&')
end)

-- The history: `:echomsg` and errors land in it; `:echo` does not;
-- `:messages clear`, a count, and 'msghistory' bound it.
section('s3-history', function()
  plain()
  cap('messages initial', 'messages')
  for i = 1, 5 do
    run('echomsg "hist ' .. i .. '"')
  end
  run('echo "not in history"')
  run('silent! call nosuch()')
  run('echomsg "multi\\nline"')
  run('echomsg [1, {"a": 2}] 0z0102 1.5')
  run('echomsg "ctrl\\x01\\x7f" "\\u00e9"')
  cap('messages all', 'messages')
  cap('messages 2', '2messages')
  cap('messages 0', '0messages')
  cap('messages 100', '100messages')
  cap('execute messages', 'echo execute("messages")')
  cap('messages clear', 'messages clear')
  cap('messages after clear', 'messages')
  cap('messages bad arg', 'messages nonsense')
  run('set msghistory=3')
  for i = 1, 6 do
    run('echomsg "cap ' .. i .. '"')
  end
  cap('msghistory=3', 'messages')
  run('set msghistory=1')
  cap('msghistory=1', 'messages')
  run('set msghistory=0')
  run('echomsg "with zero"')
  cap('msghistory=0', 'messages')
  run('set msghistory&')
  for i = 1, 3 do
    run('echomsg "after reset ' .. i .. '"')
  end
  cap('messages 1 clear', '1messages clear')
  cap('messages after count clear', 'messages')
  ev('getmessages lua', 'luaeval("vim.api.nvim_exec2(\\"messages\\", {output = true}).output")')
end)

-- Capture: `:redir` to a variable, a register and a file, nesting, and
-- `execute()` in every silence mode.
section('s4-capture', function()
  plain()
  run('redir => g:r1 | echo "one" | echon "two" | echomsg "three" | redir END')
  ev('redir var', 'g:r1')
  run('redir =>> g:r1 | echo "more" | redir END')
  ev('redir var append', 'g:r1')
  run('redir @a | echo "to reg" | redir END')
  ev('redir reg', 'getreg("a")')
  run('redir @A | echo "appended" | redir END')
  ev('redir reg append', 'getreg("a")')
  run('redir @" | echo "unnamed" | redir END')
  ev('redir unnamed', 'getreg(\'"\')')
  run('redir! > redir.txt | echo "file" | echo [1, 2] | redir END')
  ev('redir file', 'readfile("redir.txt")')
  run('redir >> redir.txt | echo "file2" | redir END')
  ev('redir file append', 'readfile("redir.txt")')
  cap('redir file exists', 'redir > redir.txt')
  run('redir END')
  run('redir => g:r2 | silent echo "silent in redir" | redir END')
  ev('redir silent', 'g:r2')
  run('redir => g:r3 | call nosuchfunc() | redir END')
  ev('redir error', 'g:r3')
  run('redir => g:outer | echo "o1" | redir => g:inner | echo "i1" | redir END | echo "o2" | redir END')
  ev('redir nested outer', 'g:outer')
  ev('redir nested inner', 'g:inner')
  run('redir => g:r4 | echo execute("echo \\"inside\\"") | redir END')
  ev('redir around execute', 'g:r4')
  for _, e in ipairs({
    'execute("echo 1")', 'execute("echo 1", "")', 'execute("echo 1", "silent")', 'execute("echo 1", "silent!")',
    'execute(["echo 1", "echo 2"])', 'execute("echon \\"x\\"")', 'execute("echomsg \\"m\\"")',
    'execute("call nosuch()", "silent!")', 'execute("")', 'execute([])', 'execute("echo 1", "bad")',
    'execute("echo execute(\\"echo 2\\")")', 'execute("redir => g:x | echo 3 | redir END") . g:x',
    'execute("throw \\"t\\"", "silent!")', 'execute("echo \\"a\\nb\\"")', 'execute("echo repeat(\\"z\\", 300)")',
    'execute("set tabstop?")', 'execute("verbose set tabstop?")', 'execute("echo \\"\\x01\\x80\\"")',
  }) do
    ev(e, e)
  end
  cap('execute in try', 'try | call execute("call nosuch()") | catch | echo v:exception | endtry')
  cap('execute error', 'echo execute("call nosuch()")')
  msgvars('vars end s4')
end)

-- `str2special`/`msg_outtrans_special`: the mapping listing, `keytrans()`,
-- and `:echo` of keys and control bytes.
section('s5-special', function()
  plain()
  run('nnoremap <F1> <C-a><Tab>x<Esc>')
  run('nnoremap <buffer> <M-x> <Cmd>echo "\\u00e9"<CR>')
  run('inoremap <C-k>a \\x80\\xfe<Space>')
  run('nnoremap <Leader>l <lt>lt><Bar><Bslash>')
  cap('nmap listing', 'nmap')
  cap('imap listing', 'imap')
  cap('map F1', 'map <F1>')
  for _, e in ipairs({
    '"\\<C-a>"', '"\\<Tab>\\<CR>\\<Esc>"', '"\\<F1>\\<S-F2>\\<C-Home>"', '"\\<M-x>\\<D-y>"',
    '"\\x80\\xfd\\x01"', '"\\u00e9 \\u65e5"', '"<lt>"', '" "', '"\\x7f"', '"\\<Nul>"',
  }) do
    ev('keytrans ' .. e, 'keytrans(' .. e .. ')')
    cap('echo key ' .. e, 'echo ' .. e)
  end
  ev('strtrans', 'strtrans("a\\x01\\x7f\\x80\\u00e9\\t")')
  cap('echo control chars', 'echo "\\x01\\x02\\x1b\\x1f\\x7f"')
  cap('echo tab', 'echo "a\\tb"')
  cap('echo invalid utf8', 'echo "\\xc3\\x28 \\xe2\\x82 \\xf0\\x9f\\x98"')
  cap('echo combining', 'echo "e\\u0301 a\\u0300\\u0316"')
  cap('echo wide', 'echo "\\u65e5\\u672c\\u8a9e"')
  cap('echo emoji', 'echo "\\U0001F600 \\U0001F468\\u200d\\U0001F469"')
  cap('echo display=uhex', 'set display=uhex | echo "\\x01\\x80" | set display&')
  cap('echo isprint', 'set isprint=1-255 | echo "\\x01\\x80" | set isprint&')
end)

-- ================================================================ screen

-- `:echo` drawn: a message that fits, one that wraps, the hit-enter
-- prompt, the more prompt, `:echon`, `:echohl`, `:echoerr`.
section('s6-screen-echo', function()
  start(40, 8)
  --- One case from a clean command line: leave any prompt, clear the
  --- screen of the last case's message, type, look.
  local function shot(name, keys)
    settle()
    feed(':redraw!<CR>')
    feed(keys)
    snap(name)
  end
  shot('echo fits', ':echo "hello world"<CR>')
  shot('echo wraps', ':echo repeat("x", 60)<CR>')
  shot('echo two lines hit-enter', ':echo "one\\ntwo"<CR>')
  feed('<CR>')
  snap('after hit-enter')
  shot('echo then colon at hit-enter', ':echo "one\\ntwo"<CR>')
  feed(':')
  snap('colon at hit-enter')
  shot('echon', ':echon "a" | echon "b"<CR>')
  shot('echohl ErrorMsg', ':echohl ErrorMsg | echo "red" | echohl None<CR>')
  shot('echohl mixed', ':echohl WarningMsg | echon "warn" | echohl Question | echon "q" | echohl None<CR>')
  shot('echohl unknown', ':echohl NoSuchGroup | echo "nosuch" | echohl None<CR>')
  shot('echohl echomsg', ':echohl ErrorMsg | echomsg "red hist" | echohl None<CR>')
  shot('echoerr', ':echoerr "an error"<CR>')
  shot('emsg E117', ':call nosuch()<CR>')
  shot('two errors', ':call nosuch() | call nosuch2()<CR>')
  shot('error in function', ':function! F()\ncall nosuch()\nendfunction<CR>:call F()<CR>')
  shot('echo control and wide', ':echo "\\x01\\x7f\\x80 \\u00e9\\u65e5\\t|"<CR>')
  shot('echo combining emoji', ':echo "ab\\u0301c \\U0001F600"<CR>')
  shot('echo containers', ':echo [1, {"a": 1.5}, 0z01, function("tr")]<CR>')
  shot('echo wide wraps', ':echo repeat("\\u65e5", 25)<CR>')
  settle()
  feed(':set more<CR>')
  shot('more prompt', ':for i in range(20) | echo "line" i | endfor<CR>')
  feed('j')
  snap('more j')
  feed('f')
  snap('more f')
  feed('b')
  snap('more b')
  feed('G')
  snap('more G')
  feed('q')
  snap('more q')
  shot('more then q at once', ':for i in range(20) | echo "line" i | endfor<CR>')
  feed(':')
  snap('more colon')
  settle()
  feed(':set nomore<CR>')
  shot('nomore', ':for i in range(20) | echo "line" i | endfor<CR>')
  shot('messages drawn', ':messages<CR>')
  settle()
end)

-- `msg_trunc`: `shortmess` T/t and 'cmdheight' at several widths.  A file
-- message is truncated at its start (`t`), any other at its middle (`T`),
-- and only when it would otherwise scroll.
section('s7-screen-trunc', function()
  local file = 'a-rather-long-file-name-for-truncation-tests.txt'
  for _, cols in ipairs({ 20, 40, 80 }) do
    start(cols, 6)
    n.command('call writefile(["x"], "' .. file .. '")')
    for _, sm in ipairs({ 'T', 't', 'tT', '' }) do
      for _, ch in ipairs({ 1, 2 }) do
        local tag = ('c%d sm=%s ch=%d'):format(cols, sm, ch)
        n.command('set shortmess=I' .. sm .. ' cmdheight=' .. ch)
        feed(':echomsg repeat("m", ' .. (cols + 15) .. ')<CR>')
        snap(tag .. ' echomsg')
        settle()
        feed(':edit! ' .. file .. '<CR>')
        snap(tag .. ' file msg')
        settle()
        feed(':call nosuch_' .. string.rep('f', cols) .. '()<CR>')
        snap(tag .. ' long emsg')
        settle()
        feed(':file<CR>')
        snap(tag .. ' file info')
        settle()
        feed('/' .. string.rep('p', cols + 10) .. '<CR>')
        snap(tag .. ' search echo')
        settle()
      end
    end
    n.command('set cmdheight=0 shortmess=I')
    feed(':echo "with cmdheight zero"<CR>')
    snap(('c%d ch=0 echo'):format(cols))
    settle()
    feed(':echo repeat("z", ' .. (cols * 2) .. ')<CR>')
    snap(('c%d ch=0 long'):format(cols))
    settle()
    n.command('set cmdheight=3')
    feed(':echo "a\\nb\\nc"<CR>')
    snap(('c%d ch=3 three lines'):format(cols))
    settle()
    feed(':echo "a\\nb\\nc\\nd"<CR>')
    snap(('c%d ch=3 four lines'):format(cols))
    settle()
    n.command('set cmdheight=1')
    ev(('c%d messages'):format(cols), 'execute("messages")')
  end
end)

-- `confirm()` with scripted answers: a hotkey, <CR> (the default), <Esc>
-- (0), a key that is no button (re-asks), the dialog types, many buttons
-- at a narrow width.
section('s8-confirm', function()
  start(40, 10)
  local cases = {
    { 'hotkey n', 'confirm("Save?", "&Yes\\n&No\\n&Cancel")', 'n' },
    { 'enter default', 'confirm("Save?", "&Yes\\n&No\\n&Cancel", 2)', '<CR>' },
    { 'escape', 'confirm("Save?", "&Yes\\n&No")', '<Esc>' },
    { 'uppercase', 'confirm("Pick", "&Apple\\n&Banana")', 'B' },
    { 'no hotkey', 'confirm("Pick", "Apple\\nBanana")', 'b' },
    { 'bad key then y', 'confirm("Save?", "&Yes\\n&No")', 'zy' },
    { 'type Error', 'confirm("Bad", "&Ok", 1, "Error")', 'o' },
    { 'type Warning', 'confirm("Hmm", "&Ok", 1, "Warning")', 'o' },
    { 'type Info', 'confirm("FYI", "&Ok", 1, "Info")', 'o' },
    { 'type Question', 'confirm("Q?", "&Ok", 1, "Question")', 'o' },
    { 'default only', 'confirm("Just ok")', '<CR>' },
    { 'multiline msg', 'confirm("line one\\nline two", "&Yes\\n&No")', 'y' },
    { 'many buttons', 'confirm("Which", "&One\\n&Two\\nT&hree\\n&Four\\nF&ive\\n&Six\\nSe&ven\\n&Eight")', 'e' },
    { 'multibyte hotkey', 'confirm("Mb", "&\\u00e9t\\u00e9\\n&Hiver")', '\u{00e9}' },
    { 'default out of range', 'confirm("Def", "&A\\n&B", 9)', '<CR>' },
  }
  for _, c in ipairs(cases) do
    n.command('let g:answer = -1')
    feed(':let g:answer = ' .. c[2] .. '<CR>')
    snap('confirm ' .. c[1])
    feed(c[3])
    -- A key that answers nothing leaves the dialog up, and a request
    -- other than a FAST one would wait for it: settle first.
    local after = n.api.nvim_get_mode()
    settle()
    ev('confirm ' .. c[1] .. ' answer (mode after key ' .. after.mode .. ')', 'g:answer')
  end
  ev('confirm no args', 'confirm()')
end)

-- `nvim_echo` chunks and every message's kind, under `ext_messages`.
section('s9-ext-messages', function()
  start(40, 8, { ext_messages = true })
  local api_cases = {
    { 'plain', { { 'plain' } }, false, {} },
    { 'hl chunks', { { 'err', 'ErrorMsg' }, { ' and ' }, { 'warn', 'WarningMsg' } }, true, {} },
    { 'hl unknown', { { 'x', 'NoSuchGroup' } }, true, {} },
    { 'err opt', { { 'as error' } }, true, { err = true } },
    { 'verbose opt', { { 'verbose only' } }, true, { verbose = true } },
    { 'kind', { { 'kinded' } }, true, { kind = 'echomsg' } },
    { 'multiline', { { 'a\nb' } }, true, {} },
    { 'empty', { { '' } }, true, {} },
    { 'progress', { { 'working' } }, true, { kind = 'progress', title = 'job', status = 'running', percent = 40, source = 'msgsweep' } },
  }
  -- The harness keeps every `msg_show` it was sent; each case starts
  -- from none so a row says what THAT case showed.
  for _, c in ipairs(api_cases) do
    screen.messages = {}
    local tag = case('nvim_echo ' .. c[1])
    local ok, res = pcall(n.api.nvim_echo, c[2], c[3], c[4])
    say(vals, tag, ok and (' = ' .. esc(ins(res))) or (' ! ' .. esc(errtext(res))))
    snap('nvim_echo ' .. c[1] .. ' screen')
  end
  ev('nvim_echo messages', 'execute("messages")')
  local typed = {
    { 'echo', ':echo "e"<CR>' },
    { 'echon', ':echon "n"<CR>' },
    { 'echomsg', ':echomsg "m"<CR>' },
    { 'echoerr', ':echoerr "x"<CR>' },
    { 'emsg', ':call nosuch()<CR>' },
    { 'echohl', ':echohl ErrorMsg | echo "h" | echohl None<CR>' },
    { 'lua print', ':lua print("p")<CR>' },
    { 'lua error', ':lua error("boom")<CR>' },
    { 'set query', ':set tabstop?<CR>' },
    { 'file info', ':file<CR>' },
    { 'list cmd', ':ls<CR>' },
    { 'wmsg', ':call setline(1, ["a", "a"]) | 2 | exe "normal! /a\\r"<CR>' },
    { 'return prompt', ':echo "a\\nb"<CR>' },
    { 'confirm', ':call confirm("Q", "&Yes")<CR>' },
  }
  for _, c in ipairs(typed) do
    screen.messages = {}
    feed(c[2])
    snap('kind ' .. c[1])
    settle()
  end
  screen.messages = {}
  feed(':messages<CR>')
  snap('kind messages history')
  settle()
  msgvars('vars end s9')
end)

-- ============================================================== reports

-- The reports the editor assembles in a scratch line before showing it
-- (upstream's `IObuff`/`msg_buf`): `msgmore`, `:print`/`:list` under
-- 'listchars', `:ascii`, `:tabs`, `:command`, `:sign`, `:history`, the
-- `:set` rejection, an error's "Error detected while processing"/"line N:"
-- source lines from a sourced script, and `keytrans()`/maparg() over the
-- `str2special` rendering.
section('s10-reports', function()
  plain()
  run('set report=0')
  run('call setline(1, range(1, 12))')
  cap('msgmore fewer', '3,5d')
  cap('msgmore one less', '1d')
  cap('msgmore more', 'normal! 3yyP')
  cap('msgmore one more', 'normal! yyp')
  cap('msgmore undo', 'undo')
  run('set report=5')
  cap('msgmore under report', '2,3d')
  run('set report&')
  run('%d _')
  run('call setline(1, ["\tlead tab", "  two  spaces  ", "x y z", "ctl\x01\x80\x7f",'
    .. ' "wide 日本 é", "    ", "", "a\t\tb", "trail\t "])')
  for _, lcs in ipairs({
    'eol:$', 'tab:>-,trail:~', 'tab:<->,space:.', 'nbsp:+,eol:$', 'multispace:ab,lead:L',
    'leadmultispace:xy,tab:>-', 'tab:>-,leadtab:+-', 'trail:-,space:_,eol:¶',
  }) do
    run('set listchars=' .. lcs)
    cap('list ' .. lcs, '%list')
    cap('print ' .. lcs, '%print')
    cap('number list ' .. lcs, 'set list | %number | set nolist')
  end
  run('set listchars&')
  for _, ch in ipairs({ 'a', '\\x01', '\\x7f', '\\x80', '\\u00e9', '\\u65e5', 'e\\u0301\\u0316',
    '\\U0001F600', '\\u00a0', '\\t', ' ', '\\u2026' }) do
    run('call setline(1, "' .. ch .. '") | 1 | normal! 0')
    cap('ascii ' .. ch, 'ascii')
  end
  run('tabnew | file one | tabnew | vsplit | file two | call setline(1, "x") | tabnew')
  cap('tabs', 'tabs')
  run('tabonly! | only!')
  run('command! -bang -bar -count=3 -nargs=* -complete=file Aaa echo 1')
  run('command! -range=% -nargs=* -addr=lines Bbb echo 2')
  run('command! -buffer -nargs=1 -complete=customlist,Foo Ccc echo 3')
  run('command! LongCommandNameThatPushesTheColumns echo "' .. string.rep('d', 60) .. '"')
  run('command! -register -nargs=? -range Ddd echo <q-args>')
  cap('command list', 'command')
  cap('command one', 'command Bbb')
  cap('command none', 'command Zzz')
  run('sign define S1 text=>> texthl=Error linehl=Search')
  run('sign define S2 text=é! numhl=Todo priority=20')
  run('call setline(1, ["a", "b", "c"])')
  run('exe "sign place 5 line=2 name=S1 buffer=" .. bufnr()')
  run('exe "sign place 6 line=3 group=grp name=S2 priority=30 buffer=" .. bufnr()')
  cap('sign list', 'sign list')
  cap('sign list one', 'sign list S2')
  cap('sign place listing', 'sign place')
  cap('sign place group', 'sign place group=*')
  cap('sign unknown', 'sign list Nope')
  run('call sign_unplace("*") | sign undefine S1 | sign undefine S2')
  for _, h in ipairs({ 'echo "short"', 'echo "' .. string.rep('h', 120) .. '"',
    'echo "' .. string.rep('\\u65e5', 60) .. '"', 'echo "mixed \\u00e9\\u0301 ' .. string.rep('m', 90) .. '"' }) do
    run("call histadd(':', '" .. h:gsub("'", "''") .. "')")
  end
  cap('history', 'history')
  cap('history range', 'history : -2,')
  for _, a in ipairs({ 'fileformat=bogus', 'tabstop=-1', 'display=\\x01x', 'nosuchopt=1' }) do
    cap('set reject ' .. a, 'exe "set ' .. a .. '"')
  end
  run('call writefile(["echo \\"one\\"", "call nosuch()", "echo \\"two\\"", "call nosuch2()"], "errs.vim")')
  cap('source errors', 'source errs.vim')
  run('redir => g:srcerr | silent! source errs.vim | redir END')
  ev('source errors silent redir', 'g:srcerr')
  run('redir => g:srcerr | silent! source errs.vim | redir END')
  ev('source errors silent redir again', 'g:srcerr')
  msgvars('vars after source errors')
  run('nnoremap <S-F3> :<C-u>echo "\\u00e9"<CR><Space><lt>')
  run('nnoremap <expr> <C-k>z "x\\<Tab>"')
  for _, e in ipairs({ 'maparg("<S-F3>", "n")', 'maparg("<S-F3>", "n", 0, 1).lhs',
    'maparg("<S-F3>", "n", 0, 1).rhs', 'maparg("<C-k>z", "n", 0, 1)', 'maplist()->len()',
    'map(maplist(), "v:val.lhs")', 'keytrans("\\<S-F3>\\<M-\\u00e9>")' }) do
    ev(e, e)
  end
  cap('map listing special', 'nmap <S-F3>')
end)

-- The same reports where the screen shapes them: `:s///c`'s prompt, the
-- `v:scrollstart` a scrolled message leaves, and `:history` and `:tabs`
-- cut to a narrow screen.
section('s11-screen-reports', function()
  start(40, 8)
  n.command('call setline(1, ["one apple", "two apples"])')
  feed(':%s/apple/pear/gc<CR>')
  snap('substitute confirm prompt')
  feed('y')
  snap('substitute confirm after y')
  feed('q')
  settle()
  ev('substitute confirm result', 'getline(1, "$")')
  n.command('let v:scrollstart = ""')
  feed(':for i in range(12) | echo "scroll" i | endfor<CR>')
  snap('scrolled')
  settle()
  ev('scrollstart typed', 'v:scrollstart')
  n.command('function! Scroller()\nfor i in range(12)\necho "f" i\nendfor\nendfunction')
  n.command('let v:scrollstart = ""')
  feed(':call Scroller()<CR>')
  settle()
  ev('scrollstart function', 'v:scrollstart')
  n.command('call histadd(":", "echo \\"' .. string.rep('\\u00e9', 50) .. '\\"")')
  n.command('call histadd(":", "echo \\"' .. string.rep('k', 70) .. '\\"")')
  feed(':history<CR>')
  snap('history narrow')
  settle()
  feed(':file ' .. string.rep('n', 30) .. '<CR>')
  settle()
  feed(':tabnew<CR>:tabs<CR>')
  snap('tabs narrow')
  settle()
  n.command('call writefile(["echo \\"one\\"", "call nosuch()", "echo \\"two\\"", "call nosuch2()"], "errs2.vim")')
  feed(':source errs2.vim<CR>')
  snap('source errors drawn')
  settle()
  feed(':source errs2.vim<CR>')
  snap('source errors drawn again')
  settle()
end)

-- ------------------------------------------------------------------- run

local only = os.getenv('MSGSWEEP_ONLY')
if only == '' then
  only = nil
end
for _, s in ipairs(S) do
  if not only or s.name:match(only) then
    if trace then
      io.stderr:write('msgsweep: ', s.name, '\n')
    end
    say(vals, '## ', s.name)
    say(txt, '## ', s.name)
    local ok, err = pcall(s.fn)
    if not ok then
      say(txt, '## ', s.name, ' SECTION ERROR ', esc(errtext(err)))
      say(vals, '## ', s.name, ' SECTION ERROR ', esc(errtext(err)))
    end
    pcall(n.check_close)
  end
end
say(vals, '## LAST ', tostring(case_no))

local function dump(path, t)
  local f = assert(io.open(path, 'w'))
  f:write(table.concat(t, '\n'), '\n')
  f:close()
end
dump(('%s/%s.txt'):format(outdir, label), txt)
dump(('%s/%s.attrs'):format(outdir, label), attrs)
dump(('%s/%s.vals'):format(outdir, label), vals)
io.write(('msgsweep %s: %d txt / %d attrs / %d vals lines, %d cases\n'):format(
  label, #txt, #attrs, #vals, case_no))
