use serde::Serialize;
use std::collections::{HashMap, VecDeque};

const MAX_CC_ERRORS: usize = 10_000;

#[derive(Debug, Clone, Serialize)]
pub struct CcError {
    pub packet_index: u64,
    pub pid: u16,
    pub expected: u8,
    pub got: u8,
}

#[derive(Debug, Default)]
pub struct ContinuityChecker {
    last_cc: HashMap<u16, u8>,
    errors: VecDeque<CcError>,
}

impl ContinuityChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check(&mut self, pid: u16, cc: u8, has_payload: bool, packet_index: u64) {
        if !has_payload || pid == 0x1FFF {
            return;
        }

        if let Some(&last) = self.last_cc.get(&pid) {
            let expected = (last + 1) & 0x0F;
            if cc != expected && cc != last {
                if self.errors.len() >= MAX_CC_ERRORS {
                    self.errors.pop_front();
                }
                self.errors.push_back(CcError {
                    packet_index,
                    pid,
                    expected,
                    got: cc,
                });
            }
        }

        self.last_cc.insert(pid, cc);
    }

    pub fn errors(&self) -> &VecDeque<CcError> {
        &self.errors
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn recent_errors(&self, count: usize) -> Vec<&CcError> {
        self.errors.iter().rev().take(count).collect()
    }
}
