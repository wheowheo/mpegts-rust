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

#[allow(dead_code)]
pub struct SystemStatsCollector {
    start: Instant,
    last_cpu_times: Option<(f64, f64)>,
    last_net_tx: Option<(u64, Instant)>,
}

impl SystemStatsCollector {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            last_cpu_times: None,
            last_net_tx: None,
        }
    }

    pub fn snapshot(&mut self) -> SystemSnapshot {
        let timestamp = self.start.elapsed().as_secs_f64();

        SystemSnapshot {
            timestamp_sec: timestamp,
            cpu_usage_pct: self.read_cpu_usage(),
            memory_used_bytes: self.read_memory_used(),
            memory_total_bytes: self.read_memory_total(),
            net_tx_bytes_sec: 0.0, // platform-specific, filled below
            net_tx_drops: 0,
        }
    }

    #[cfg(target_os = "macos")]
    fn read_cpu_usage(&mut self) -> f64 {
        use std::process::Command;
        // ps로 현재 프로세스 CPU 사용률 조회
        let pid = std::process::id();
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "%cpu="])
            .output();
        match output {
            Ok(out) => {
                String::from_utf8_lossy(&out.stdout)
                    .trim()
                    .parse::<f64>()
                    .unwrap_or(0.0)
            }
            Err(_) => 0.0,
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn read_cpu_usage(&mut self) -> f64 {
        // /proc/self/stat 기반 계산
        if let Ok(stat) = std::fs::read_to_string("/proc/self/stat") {
            let parts: Vec<&str> = stat.split_whitespace().collect();
            if parts.len() > 14 {
                let utime: f64 = parts[13].parse().unwrap_or(0.0);
                let stime: f64 = parts[14].parse().unwrap_or(0.0);
                let total_time = utime + stime;
                let clock_ticks = 100.0; // sysconf(_SC_CLK_TCK)

                if let Some((prev_total, prev_wall)) = self.last_cpu_times {
                    let elapsed_wall = self.start.elapsed().as_secs_f64();
                    let dt_wall = elapsed_wall - prev_wall;
                    if dt_wall > 0.0 {
                        let dt_cpu = (total_time - prev_total) / clock_ticks;
                        let usage = (dt_cpu / dt_wall) * 100.0;
                        self.last_cpu_times = Some((total_time, elapsed_wall));
                        return usage;
                    }
                }
                self.last_cpu_times = Some((total_time, self.start.elapsed().as_secs_f64()));
            }
        }
        0.0
    }

    #[cfg(target_os = "macos")]
    fn read_memory_used(&self) -> u64 {
        use std::process::Command;
        let pid = std::process::id();
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "rss="])
            .output();
        match output {
            Ok(out) => {
                String::from_utf8_lossy(&out.stdout)
                    .trim()
                    .parse::<u64>()
                    .unwrap_or(0)
                    * 1024 // KB → bytes
            }
            Err(_) => 0,
        }
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
        {
            use std::process::Command;
            let output = Command::new("sysctl")
                .args(["-n", "hw.memsize"])
                .output();
            match output {
                Ok(out) => {
                    String::from_utf8_lossy(&out.stdout)
                        .trim()
                        .parse::<u64>()
                        .unwrap_or(0)
                }
                Err(_) => 0,
            }
        }
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
