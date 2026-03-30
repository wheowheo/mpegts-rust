use ts_core::pid::PidMap;

pub struct StreamAnalyzer {
    pub pid_map: PidMap,
}

impl StreamAnalyzer {
    pub fn new() -> Self {
        Self {
            pid_map: PidMap::new(),
        }
    }
}

impl Default for StreamAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
