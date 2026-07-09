#![allow(dead_code)]

use crate::config::PermissionMode;
use crate::executor::actions::ToolAction;

pub struct SecurityValidator {
    permission_mode: PermissionMode,
}

impl SecurityValidator {
    pub fn new(permission_mode: PermissionMode) -> Self {
        Self { permission_mode }
    }

    pub fn validate_tool_action(&self, action: &ToolAction) -> Result<(), SecurityError> {
        match action {
            ToolAction::ExecuteCommand { command } => {
                if !self.is_operator() {
                    return Err(SecurityError::InsufficientPermissions {
                        required: "Operator".to_string(),
                        current: "Player".to_string(),
                    });
                }

                let dangerous_commands = vec![
                    "op",
                    "deop",
                    "ban",
                    "pardon",
                    "kick",
                    "stop",
                    "restart",
                    "whitelist",
                    "gamemode",
                    "difficulty",
                    "time set",
                    "weather",
                ];

                let cmd_lower = command.to_lowercase();
                for dangerous in &dangerous_commands {
                    if cmd_lower.starts_with(dangerous) {
                        return Err(SecurityError::DangerousCommand {
                            command: command.clone(),
                        });
                    }
                }

                Ok(())
            }
            ToolAction::GiveItem { .. } => {
                if !self.is_operator() {
                    return Err(SecurityError::InsufficientPermissions {
                        required: "Operator".to_string(),
                        current: "Player".to_string(),
                    });
                }
                Ok(())
            }
            ToolAction::Teleport { .. } => {
                if !self.is_operator() {
                    return Err(SecurityError::InsufficientPermissions {
                        required: "Operator".to_string(),
                        current: "Player".to_string(),
                    });
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn validate_chat_message(&self, message: &str) -> Result<(), SecurityError> {
        if message.len() > 256 {
            return Err(SecurityError::MessageTooLong {
                length: message.len(),
                max: 256,
            });
        }

        if message.contains('\n') || message.contains('\r') {
            return Err(SecurityError::InvalidCharacter);
        }

        Ok(())
    }

    pub fn sanitize_input(&self, input: &str) -> String {
        input
            .chars()
            .filter(|c| !c.is_control() || *c == '\n')
            .take(256)
            .collect()
    }

    fn is_operator(&self) -> bool {
        matches!(self.permission_mode, PermissionMode::Operator)
    }
}

#[derive(Debug, Clone)]
pub enum SecurityError {
    InsufficientPermissions { required: String, current: String },
    DangerousCommand { command: String },
    MessageTooLong { length: usize, max: usize },
    InvalidCharacter,
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::InsufficientPermissions { required, current } => {
                write!(
                    f,
                    "Insufficient permissions: {} required, have {}",
                    required, current
                )
            }
            SecurityError::DangerousCommand { command } => {
                write!(f, "Dangerous command blocked: /{}", command)
            }
            SecurityError::MessageTooLong { length, max } => {
                write!(f, "Message too long: {} chars (max {})", length, max)
            }
            SecurityError::InvalidCharacter => {
                write!(f, "Invalid character in message")
            }
        }
    }
}

impl std::error::Error for SecurityError {}
