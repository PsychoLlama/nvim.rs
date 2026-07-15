extern "C" {
    fn uv_get_total_memory() -> uint64_t;
}
pub type uint64_t = u64;
#[no_mangle]
pub unsafe extern "C" fn os_get_total_mem_kib() -> uint64_t {
    return uv_get_total_memory().wrapping_div(1024 as uint64_t);
}
