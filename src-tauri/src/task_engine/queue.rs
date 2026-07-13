use std::collections::VecDeque;
use std::sync::Arc;

use parking_lot::RwLock;
use uuid::Uuid;

use super::types::{Task, TaskPriority};

#[derive(Debug, Clone)]
pub struct QueuedTask {
    pub id: String,
    pub task: Task,
    pub priority: TaskPriority,
}

pub struct TaskQueue {
    tasks: VecDeque<QueuedTask>,
    max_size: usize,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
            max_size: 100,
        }
    }

    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            tasks: VecDeque::new(),
            max_size,
        }
    }

    pub fn enqueue(&mut self, task: Task) -> String {
        let id = Uuid::new_v4().to_string();
        let priority = task.priority();

        let queued = QueuedTask {
            id: id.clone(),
            task,
            priority,
        };

        if self.tasks.len() >= self.max_size {
            self.tasks.pop_back();
        }

        let insert_pos = self.tasks.iter().position(|t| t.priority > priority);
        match insert_pos {
            Some(pos) => self.tasks.insert(pos, queued),
            None => self.tasks.push_back(queued),
        }

        id
    }

    pub fn dequeue(&mut self) -> Option<QueuedTask> {
        self.tasks.pop_front()
    }

    pub fn peek(&self) -> Option<&QueuedTask> {
        self.tasks.front()
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn remove(&mut self, id: &str) -> Option<QueuedTask> {
        if let Some(pos) = self.tasks.iter().position(|t| t.id == id) {
            self.tasks.remove(pos)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.tasks.clear();
    }

    pub fn get_tasks(&self) -> Vec<&QueuedTask> {
        self.tasks.iter().collect()
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedTaskQueue = Arc<RwLock<TaskQueue>>;

pub fn new_shared_queue() -> SharedTaskQueue {
    Arc::new(RwLock::new(TaskQueue::new()))
}
