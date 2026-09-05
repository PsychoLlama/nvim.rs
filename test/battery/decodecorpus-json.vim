" JSON decode hand-differential: drive json_decode() over a corpus aimed at
" the scanner's error arms, the escape/surrogate paths and the special-map
" restart, then re-encode what came back.
let s:cases = [
\ '', ' ', "\t\n\r ", 'null', 'true', 'false', 'nul', 'tru', 'fals', 'nullx', 'truex',
\ 'n', 't', 'f', '"', '""', '"a"', '"\\"', '"\/"', '"\b\f\n\r\t"', '"\q"', '"\u"',
\ '"A"', '"😀"', '"\uD83D"', '"\uDE00"', '"\uD83Dx"', '"\uD83DA"',
\ '"\uD83D😀"', '"\ud800"', '"\udfff"', '" "', '"a b"',
\ '"AB"', '"￿"', '"￾"', '"퟿"', '""',
\ '"abc', '"ab\', '"a' . nr2char(1) . 'b"', '"' . nr2char(127) . '"',
\ '"' . "\xff" . '"', '"' . "\xc3\x83" . '"', '"' . "\xc3" . '"',
\ '"' . "\xfc\x90\x80\x80\x80\x80" . '"', '"' . "\xed\xa0\x80" . '"',
\ '"' . "\xf4\x90\x80\x80" . '"', '"' . "\xf0\x9f\x98\x80" . '"',
\ '0', '-0', '1', '-1', '00', '01', '-01', '1.', '1.0', '-1.5', '1e', '1e5', '1E5',
\ '1e+5', '1e-5', '1.0e', '1.0e+', '1.0e-', '1.e5', '.5', '-', '-.', '1.2.3',
\ '9223372036854775807', '9223372036854775808', '-9223372036854775808',
\ '-9223372036854775809', '18446744073709551616', '1e400', '-1e400', '1e-400',
\ '0.1', '1.7976931348623157e308', '5e-324', '0e0', '-0.0',
\ '[]', '[1]', '[1,2]', '[1,]', '[,1]', '[,]', '[1 2]', '[', ']', '[[]]', '[[1],[2]]',
\ '[1,2,3,]', '[}', '{]', '{}', '{"a":1}', '{"a":1,"b":2}', '{"a":1,}', '{,"a":1}',
\ '{"a"}', '{"a":}', '{:1}', '{"a"::1}', '{"a",1}', '{"a":1,"a":2}', '{"":1}',
\ '{" ":1}', '{1:2}', '{[]:1}', '{null:1}', '{"a":{"b":{"c":1}}}',
\ '{"a":[1,{"b":2}]}', '{"_TYPE":[],"_VAL":[]}', '{"a":1}x', '  {"a":1}  ',
\ '[1,2] [3]', '{"a":1}{"b":2}', '1 2', '{"a":1,"a":2,"c":3}',
\ '{"a":1,"b":{"c":1,"c":2}}', '[{"a":1,"a":2}]', '{"a b":1}',
\ '{"a":1,"a b":2}', '{"x":1,"y":2,"x":3,"z":4}',
\ '[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[1]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]',
\ '?', '@', nr2char(1), '"a"' . nr2char(9) . 'b', '[1' . nr2char(11) . ']',
\ ]
for c in s:cases
  let out = ''
  try
    let out = string(json_decode(c))
  catch
    let out = 'THROW ' . v:exception
  endtry
  echo strtrans(c) . "\t=> " . out
endfor
for c in s:cases
  let out = ''
  try
    let out = json_encode(json_decode(c))
  catch
    let out = 'THROW ' . v:exception
  endtry
  echo 'rt ' . strtrans(c) . "\t=> " . strtrans(out)
endfor
