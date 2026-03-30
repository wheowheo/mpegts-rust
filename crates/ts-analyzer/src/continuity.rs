use serde::Serialize;
use std::collections::HashMap;

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
    errors: Vec<CcError>,
}

impl ContinuityChecker {
    pub fn new() -> Self {
        Self::default()
    }

    /// TS 패킷의 CC를 확인하고, 에러가 있으면 기록
    /// has_payload: adaptation_field_control의 bit 0 (payload 존재 여부)
    pub fn check(&mut self, pid: u16, cc: u8, has_payload: bool, packet_index: u64) {
        // payload가 없는 패킷은 CC가 증가하지 않으므로 skip
        if !has_payload {
            return;
        }

        // null 패킷은 CC 체크 대상 아님
        if pid == 0x1FFF {
            return;
        }

        if let Some(&last) = self.last_cc.get(&pid) {
            let expected = (last + 1) & 0x0F;
            // 동일 CC는 duplicate packet (허용)
            if cc != expected && cc != last {
                self.errors.push(CcError {
                    packet_index,
                    pid,
                    expected,
                    got: cc,
                });
            }
        }

        self.last_cc.insert(pid, cc);
    }

    pub fn errors(&self) -> &[CcError] {
        &self.errors
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn errors_for_pid(&self, pid: u16) -> Vec<&CcError> {
        self.errors.iter().filter(|e| e.pid == pid).collect()
    }

    pub fn recent_errors(&self, count: usize) -> &[CcError] {
        let start = self.errors.len().saturating_sub(count);
        &self.errors[start..]
    }
}
