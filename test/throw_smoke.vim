" Throw-from-function smoke test: exercises function-scoped throw/catch paths.
"
" Regression guard for the infinite-recursion SIGSEGV in get_scriptname:
" commit 7f4f9ecf9a left rs_get_scriptname as a trampoline calling back into C,
" creating a 3-way loop that crashed whenever a VimL function threw an exception.
"
" just check previously ran only build + smoke-test-run + smoke-test-regexp and
" never exercised function-scoped throws, so this crash survived multiple waves.
"
" IMPORTANT: This script must complete within the timeout enforced by the
" justfile target. If it hangs, the timeout will kill the process.

let s:errors = []
let s:pass_count = 0

function! s:ok(name)
  let s:pass_count += 1
endfunction

function! s:fail(name, msg)
  call add(s:errors, a:name .. ': ' .. a:msg)
endfunction

" ============================================================================
" Test 1: throw from inside a function caught by enclosing try/catch
" ============================================================================

function! s:ThrowBoom() abort
  throw 'boom'
endfunction

let s:caught = ''
try
  call s:ThrowBoom()
catch /boom/
  let s:caught = v:exception
endtry

if s:caught =~# 'boom'
  call s:ok('throw-caught-by-enclosing-try')
else
  call s:fail('throw-caught-by-enclosing-try', 'expected exception containing "boom", got: ' .. s:caught)
endif

" ============================================================================
" Test 2: throw from inside a function with no surrounding handler, suppressed
"         by silent! — the exact crash repro path (SIGSEGV before fix)
" ============================================================================

let s:sentinel = 0
silent! call s:ThrowBoom()
let s:sentinel = 1

if s:sentinel == 1
  call s:ok('throw-silent-survives')
else
  call s:fail('throw-silent-survives', 'process did not survive silent! call of throwing function')
endif

" ============================================================================
" Test 3: try/finally in a function runs finally block on throw
" ============================================================================

let s:finally_ran = 0

function! s:ThrowWithFinally() abort
  try
    throw 'inner'
  finally
    let s:finally_ran = 1
  endtry
endfunction

try
  call s:ThrowWithFinally()
catch
endtry

if s:finally_ran == 1
  call s:ok('throw-finally-runs')
else
  call s:fail('throw-finally-runs', 'finally block did not execute on throw')
endif

" ============================================================================
" Test 4: nested throw propagates across two frames into outer catch
" ============================================================================

function! s:InnerThrow() abort
  throw 'deep'
endfunction

function! s:MiddleFrame() abort
  call s:InnerThrow()
endfunction

let s:outer_caught = ''
try
  call s:MiddleFrame()
catch /deep/
  let s:outer_caught = v:exception
endtry

if s:outer_caught =~# 'deep'
  call s:ok('throw-propagates-across-frames')
else
  call s:fail('throw-propagates-across-frames', 'expected "deep" exception, got: ' .. s:outer_caught)
endif

" ============================================================================
" Report results
" ============================================================================

if len(s:errors) == 0
  echomsg 'throw-smoke: ALL ' .. s:pass_count .. ' tests passed'
else
  for err in s:errors
    echoerr 'FAIL: ' .. err
  endfor
  echoerr 'throw-smoke: ' .. len(s:errors) .. ' FAILED, ' .. s:pass_count .. ' passed'
  cquit!
endif

qall!
