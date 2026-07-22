// Nvim types come from shim.h (the C compile) or the generated cdefs chunk
// (the specs' ffi.cdef; testutil drops the '#' lines of fixture headers).

void ut_multiqueue_put(MultiQueue *self, const char *str);
const char *ut_multiqueue_get(MultiQueue *self);
