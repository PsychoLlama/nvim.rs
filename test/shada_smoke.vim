" ShaDa read/apply round-trip smoke test.
"
" Guards against the class of bug where the ShaDa read loop exits early
" (returning SD_READ_STATUS_FINISHED on the first call) due to nvim_file_eof()
" checking fp->eof without also checking fp->read_pos vs fp->write_pos.
" When HAVE_READV is defined, the first file_read() call buffers the entire
" file and sets fp->eof=true even though data remains in the buffer.
" The inline file_eof() in C correctly checks both conditions; the Rust FFI
" wrapper nvim_file_eof() previously checked only fp->eof, causing every
" entry after the first buffered read to be silently dropped.
"
" Also guards against the ex_display(:registers) SIGSEGV caused by
" rs_ex_display calling rs_buflist_name_nr and misinterpreting its return
" value (0=OK, non-zero=FAIL in Rust convention vs FAIL=0/OK=1 in C).
"
" IMPORTANT: Must complete within the timeout enforced by just smoke-test-shada
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
" Set up temp files
" ============================================================================
let s:probe_txt = tempname()
let s:shada_file = tempname() .. '.shada'

call writefile(['line1', 'line2', 'line3'], s:probe_txt)

" ============================================================================
" Test 1: Write ShaDa — mark A on line 2, register 'a' = 'HELLO'
" ============================================================================
execute 'edit ' .. s:probe_txt
normal! 2G
mark A
call setreg('a', 'HELLO')
execute 'wshada! ' .. s:shada_file
bwipeout!

" ============================================================================
" Test 2: Read ShaDa back — verify mark and register round-trip
" ============================================================================
execute 'rshada! ' .. s:shada_file

let s:marks = getmarklist()
let s:mark_a = filter(copy(s:marks), {_, v -> v.mark ==# "'A"})

if len(s:mark_a) > 0
  call s:ok('global-mark-A-restored')
else
  call s:fail('global-mark-A-restored', 'mark A not in getmarklist() after rshada; got: ' .. string(s:marks))
endif

if len(s:mark_a) > 0 && s:mark_a[0].pos[1] == 2
  call s:ok('global-mark-A-line-correct')
else
  let s:got_line = len(s:mark_a) > 0 ? s:mark_a[0].pos[1] : 'absent'
  call s:fail('global-mark-A-line-correct', 'expected line 2, got: ' .. s:got_line)
endif

let s:reg_a = getreg('a')
if s:reg_a ==# 'HELLO'
  call s:ok('register-a-restored')
else
  call s:fail('register-a-restored', "expected 'HELLO', got: " .. string(s:reg_a))
endif

" ============================================================================
" Test 3: :registers must not SIGSEGV
" ============================================================================
try
  registers
  call s:ok('registers-no-crash')
catch
  call s:fail('registers-no-crash', 'registers command threw: ' .. v:exception)
endtry

" ============================================================================
" Clean up
" ============================================================================
call delete(s:probe_txt)
call delete(s:shada_file)

" ============================================================================
" Report results
" ============================================================================
if len(s:errors) == 0
  echomsg 'shada-smoke: ALL ' .. s:pass_count .. ' tests passed'
else
  for s:err in s:errors
    echoerr 'FAIL: ' .. s:err
  endfor
  echoerr 'shada-smoke: ' .. len(s:errors) .. ' FAILED, ' .. s:pass_count .. ' passed'
  cquit!
endif

qall!
