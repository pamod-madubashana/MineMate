use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;

pub struct AuditLogger {
    logs: Arc<RwLock<Vec<AuditEntry>>>,
    max_entries: usize,
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp: String,
    pub action: String,
    pub player: Option<String>,
    pub result: AuditResult,
    pub details: String,
}

#[derive(Debug, Clone)]
pub enum AuditResult {
    Success,
    Failed,
    Blocked,
}

impl AuditLogger {
    pub fn new(max_entries: usize) -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
            max_entries,
        }
    }

    pub fn log(&self, action: &str, player: Option<&str>, result: AuditResult, details: &str) {
        let entry = AuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            action: action.to_string(),
            player: player.map(|s| s.to_string()),
            result,
            details: details.to_string(),
        };

        let mut logs = self.logs.write();
        logs.push(entry);

        if logs.len() > self.max_entries {
            logs.remove(0);
        }
    }

    pub fn log_success(&self, action: &str, player: Option<&str>, details: &str) {
        self.log(action, player, AuditResult::Success, details);
    }

    pub fn log_failure(&self, action: &str, player: Option<&str>, details: &str) {
        self.log(action, player, AuditResult::Failed, details);
    }

    pub fn log_blocked(&self, action: &str, player: Option<&str>, details: &str) {
        self.log(action, player, AuditResult::Blocked, details);
    }

    pub fn get_logs(&self) -> Vec<AuditEntry> {
        self.logs.read().clone()
    }

    pub fn get_recent_logs(&self, count: usize) -> Vec<AuditEntry> {
        let logs = self.logs.read();
        logs.iter().rev().take(count).cloned().collect()
    }

    pub fn clear_logs(&self) {
        self.logs.write().clear();
    }
}

use std::sync::OnceLock;

static AUDIT_LOGGER: OnceLock<AuditLogger> = OnceLock::new();

pub fn get_audit_logger() -> &'static AuditLogger {
    AUDIT_LOGGER.get_or_init(|| AuditLogger::new(1000))
}
