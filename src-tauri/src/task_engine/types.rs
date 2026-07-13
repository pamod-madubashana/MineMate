use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Task {
    MoveTo { x: i32, y: i32, z: i32 },
    Follow { player: String },
    Mine { block: String, count: u32 },
    Place { block: String, x: i32, y: i32, z: i32 },
    Build { blueprint: String, origin_x: i32, origin_y: i32, origin_z: i32 },
    Attack { target: String },
    Guard { player: String },
    Craft { item: String, count: u32 },
    Sleep,
    Eat,
    CollectItem { item: String, count: u32 },
    OpenChest { x: i32, y: i32, z: i32 },
    Reply { message: String },
    ExecuteCommand { command: String },
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl TaskResult {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TaskPriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
}

impl Task {
    pub fn priority(&self) -> TaskPriority {
        match self {
            Task::Attack { .. } => TaskPriority::Critical,
            Task::Guard { .. } => TaskPriority::High,
            Task::Stop => TaskPriority::Critical,
            Task::MoveTo { .. } => TaskPriority::Normal,
            Task::Follow { .. } => TaskPriority::Normal,
            Task::Mine { .. } => TaskPriority::Normal,
            Task::Place { .. } => TaskPriority::Normal,
            Task::Build { .. } => TaskPriority::Low,
            Task::Craft { .. } => TaskPriority::Normal,
            Task::Sleep => TaskPriority::Low,
            Task::Eat => TaskPriority::High,
            Task::CollectItem { .. } => TaskPriority::Normal,
            Task::OpenChest { .. } => TaskPriority::Normal,
            Task::Reply { .. } => TaskPriority::Low,
            Task::ExecuteCommand { .. } => TaskPriority::Normal,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Task::MoveTo { .. } => "move_to",
            Task::Follow { .. } => "follow",
            Task::Mine { .. } => "mine",
            Task::Place { .. } => "place",
            Task::Build { .. } => "build",
            Task::Attack { .. } => "attack",
            Task::Guard { .. } => "guard",
            Task::Craft { .. } => "craft",
            Task::Sleep => "sleep",
            Task::Eat => "eat",
            Task::CollectItem { .. } => "collect_item",
            Task::OpenChest { .. } => "open_chest",
            Task::Reply { .. } => "reply",
            Task::ExecuteCommand { .. } => "execute_command",
            Task::Stop => "stop",
        }
    }
}
