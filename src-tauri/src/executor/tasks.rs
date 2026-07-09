#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub progress: f32,
    pub assigned_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Build,
    Mine,
    Farm,
    Combat,
    Explore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

pub struct TaskQueue {
    tasks: Vec<Task>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn get_tasks(&self) -> &[Task] {
        &self.tasks
    }

    pub fn update_progress(&mut self, task_id: &str, progress: f32) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.progress = progress;
            if progress >= 1.0 {
                task.status = TaskStatus::Completed;
            }
        }
    }
}
