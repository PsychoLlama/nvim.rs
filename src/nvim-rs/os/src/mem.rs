//! System memory functions

/// Get the total system physical memory in KiB.
#[no_mangle]
pub extern "C" fn rs_os_get_total_mem_kib() -> u64 {
    #[cfg(target_os = "linux")]
    {
        // Use sysinfo on Linux
        let mut info: libc::sysinfo = unsafe { std::mem::zeroed() };
        let result = unsafe { libc::sysinfo(&mut info) };
        if result == 0 {
            // totalram is in bytes, mem_unit is the multiplier
            let total_bytes = info.totalram * u64::from(info.mem_unit);
            return total_bytes / 1024;
        }
        0
    }

    #[cfg(target_os = "macos")]
    {
        // Use sysctl on macOS
        let mut size: libc::size_t = std::mem::size_of::<u64>();
        let mut total_mem: u64 = 0;
        let mib = [libc::CTL_HW, libc::HW_MEMSIZE];
        let result = unsafe {
            libc::sysctl(
                mib.as_ptr(),
                2,
                &mut total_mem as *mut u64 as *mut libc::c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            )
        };
        if result == 0 {
            return total_mem / 1024;
        }
        0
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        // Fallback: return 0 for unsupported platforms
        0
    }
}
