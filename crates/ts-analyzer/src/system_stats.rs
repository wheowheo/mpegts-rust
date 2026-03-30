use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Clone, Serialize)]
pub struct SystemSnapshot {
    pub timestamp_sec: f64,
    pub cpu_usage_pct: f64,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub net_tx_bytes_sec: f64,
    pub net_tx_drops: u64,
}

pub struct SystemStatsCollector {
    start: Instant,
    last_cpu: Option<(u64, u64, Instant)>, // (utime_ns, stime_ns, wall)
}

impl SystemStatsCollector {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            last_cpu: None,
        }
    }

    pub fn snapshot(&mut self) -> SystemSnapshot {
        let timestamp = self.start.elapsed().as_secs_f64();

        SystemSnapshot {
            timestamp_sec: timestamp,
            cpu_usage_pct: self.read_cpu_usage(),
            memory_used_bytes: self.read_memory_used(),
            memory_total_bytes: self.read_memory_total(),
            net_tx_bytes_sec: 0.0,
            net_tx_drops: 0,
        }
    }

    #[cfg(target_os = "macos")]
    fn read_cpu_usage(&mut self) -> f64 {
        let now = Instant::now();
        let (utime_ns, stime_ns) = macos::process_cpu_times();
        let total_ns = utime_ns + stime_ns;

        let usage = if let Some((prev_total_u, prev_total_s, prev_wall)) = self.last_cpu {
            let prev_total = prev_total_u + prev_total_s;
            let wall_ns = now.duration_since(prev_wall).as_nanos() as u64;
            if wall_ns > 0 {
                (total_ns - prev_total) as f64 / wall_ns as f64 * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        self.last_cpu = Some((utime_ns, stime_ns, now));
        usage
    }

    #[cfg(not(target_os = "macos"))]
    fn read_cpu_usage(&mut self) -> f64 {
        let now = Instant::now();
        if let Ok(stat) = std::fs::read_to_string("/proc/self/stat") {
            let parts: Vec<&str> = stat.split_whitespace().collect();
            if parts.len() > 14 {
                let utime: u64 = parts[13].parse().unwrap_or(0);
                let stime: u64 = parts[14].parse().unwrap_or(0);
                let total_ticks = utime + stime;
                let clock_ticks = 100u64;

                let usage = if let Some((prev_u, prev_s, prev_wall)) = self.last_cpu {
                    let prev_total = prev_u + prev_s;
                    let wall_ns = now.duration_since(prev_wall).as_nanos() as u64;
                    if wall_ns > 0 {
                        let dt_cpu_ns = (total_ticks - prev_total) * 1_000_000_000 / clock_ticks;
                        dt_cpu_ns as f64 / wall_ns as f64 * 100.0
                    } else { 0.0 }
                } else { 0.0 };

                self.last_cpu = Some((utime, stime, now));
                return usage;
            }
        }
        0.0
    }

    #[cfg(target_os = "macos")]
    fn read_memory_used(&self) -> u64 {
        macos::process_rss_bytes()
    }

    #[cfg(not(target_os = "macos"))]
    fn read_memory_used(&self) -> u64 {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        return parts[1].parse::<u64>().unwrap_or(0) * 1024;
                    }
                }
            }
        }
        0
    }

    fn read_memory_total(&self) -> u64 {
        #[cfg(target_os = "macos")]
        { macos::total_memory_bytes() }
        #[cfg(not(target_os = "macos"))]
        {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
                for line in meminfo.lines() {
                    if line.starts_with("MemTotal:") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            return parts[1].parse::<u64>().unwrap_or(0) * 1024;
                        }
                    }
                }
            }
            0
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use std::mem;

    extern "C" {
        fn getpid() -> i32;
        fn proc_pidinfo(
            pid: i32,
            flavor: i32,
            arg: u64,
            buffer: *mut libc::c_void,
            buffersize: i32,
        ) -> i32;
    }

    // PROC_PIDTASKINFO = 4
    const PROC_PIDTASKINFO: i32 = 4;

    #[repr(C)]
    struct ProcTaskInfo {
        pti_virtual_size: u64,
        pti_resident_size: u64,
        pti_total_user: u64,   // nanoseconds
        pti_total_system: u64, // nanoseconds
        pti_threads_user: u64,
        pti_threads_system: u64,
        pti_policy: i32,
        pti_faults: i32,
        pti_pageins: i32,
        pti_cow_faults: i32,
        pti_messages_sent: i32,
        pti_messages_received: i32,
        pti_syscalls_mach: i32,
        pti_syscalls_unix: i32,
        pti_csw: i32,
        pti_threadnum: i32,
        pti_numrunning: i32,
        pti_priority: i32,
    }

    fn get_task_info() -> Option<ProcTaskInfo> {
        unsafe {
            let mut info: ProcTaskInfo = mem::zeroed();
            let size = mem::size_of::<ProcTaskInfo>() as i32;
            let ret = proc_pidinfo(
                getpid(),
                PROC_PIDTASKINFO,
                0,
                &mut info as *mut _ as *mut libc::c_void,
                size,
            );
            if ret == size { Some(info) } else { None }
        }
    }

    pub fn process_cpu_times() -> (u64, u64) {
        get_task_info()
            .map(|info| (info.pti_total_user, info.pti_total_system))
            .unwrap_or((0, 0))
    }

    pub fn process_rss_bytes() -> u64 {
        get_task_info()
            .map(|info| info.pti_resident_size)
            .unwrap_or(0)
    }

    pub fn total_memory_bytes() -> u64 {
        unsafe {
            let mut size: u64 = 0;
            let mut len = mem::size_of::<u64>();
            let name = b"hw.memsize\0";
            let ret = libc::sysctlbyname(
                name.as_ptr() as *const libc::c_char,
                &mut size as *mut _ as *mut libc::c_void,
                &mut len,
                std::ptr::null_mut(),
                0,
            );
            if ret == 0 { size } else { 0 }
        }
    }
}
