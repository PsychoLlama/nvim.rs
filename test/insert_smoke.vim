" Insert mode regression smoke test.
"
" Guards against the CPU-spin bug caused by VimState field-order mismatch
" between state/src/lib.rs (check-first) and edit/src/dispatch.rs (execute-first).
" When the fields were swapped, state_enter called execute as check and vice
" versa; insert_check_rs always returned 1 (continue), spinning infinitely on
" every key including ESC.
"
" This test exercises insert mode entry/exit via feedkeys so any spin would
" cause the 15-second just smoke-test timeout to fire.
"
" IMPORTANT: Must complete within the timeout enforced by just smoke-test-insert
" (15 seconds).  Cleans up all temp files; idempotent.

let s:errors = []
let s:pass_count = 0

function! s:ok(name)
  let s:pass_count += 1
endfunction

function! s:fail(name, msg)
  call add(s:errors, a:name .. ': ' .. a:msg)
endfunction

" ============================================================================
" Test 1: Basic insert mode entry and ESC exit
" A spin here would never return.
" ============================================================================

call feedkeys("iHello\<Esc>", 'x')

if mode() ==# 'n'
  call s:ok('esc-exits-insert-mode')
else
  call s:fail('esc-exits-insert-mode', 'after ESC mode is ' .. mode() .. ' expected n')
endif

" ============================================================================
" Test 2: Text actually inserted
" ============================================================================

if getline('.') =~# 'Hello'
  call s:ok('text-was-inserted')
else
  call s:fail('text-was-inserted', 'line is ' .. string(getline('.')) .. ' expected "Hello"')
endif

" ============================================================================
" Test 3: Multiple insert sessions with ESC each time
" ============================================================================

for s:i in range(3)
  call feedkeys("i\<Esc>", 'x')
  if mode() !=# 'n'
    call s:fail('repeated-esc-exits-insert-' .. s:i, 'mode=' .. mode())
  else
    call s:ok('repeated-esc-exits-insert-' .. s:i)
  endif
endfor

" ============================================================================
" Test 4: Insert with Ctrl-C also exits
" ============================================================================

call feedkeys("iSomeText\<C-c>", 'x')

if mode() ==# 'n'
  call s:ok('ctrl-c-exits-insert-mode')
else
  call s:fail('ctrl-c-exits-insert-mode', 'after CTRL-C mode is ' .. mode() .. ' expected n')
endif

" ============================================================================
" Test 5: New buffer, insert, ESC — exercises full insert_enter path
" ============================================================================

new
call feedkeys("iNew buffer text\<Esc>", 'x')

if mode() ==# 'n'
  call s:ok('new-buffer-esc-exits-insert')
else
  call s:fail('new-buffer-esc-exits-insert', 'mode=' .. mode())
endif

if getline('.') =~# 'New buffer text'
  call s:ok('new-buffer-text-inserted')
else
  call s:fail('new-buffer-text-inserted', 'line=' .. string(getline('.')))
endif

bdelete!

" ============================================================================
" Report results
" ============================================================================

if len(s:errors) == 0
  echomsg 'insert-smoke: ALL ' .. s:pass_count .. ' tests passed'
else
  for s:err in s:errors
    echoerr 'FAIL: ' .. s:err
  endfor
  echoerr 'insert-smoke: ' .. len(s:errors) .. ' FAILED, ' .. s:pass_count .. ' passed'
  cquit!
endif

qall!
