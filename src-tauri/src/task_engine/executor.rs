use std::sync::Arc;

use azalea::pathfinder::PathfinderClientExt;
use parking_lot::RwLock;
use tokio::sync::broadcast;

use super::queue::{SharedTaskQueue, new_shared_queue};
use super::types::{Task, TaskResult};
use crate::bot::handler::BOT_CLIENT;
use crate::bot::pathfinding;

pub struct TaskExecutor {
    queue: SharedTaskQueue,
    stop_signal: broadcast::Sender<()>,
}

impl TaskExecutor {
    pub fn new() -> Self {
        let (stop_signal, _) = broadcast::channel(1);
        Self {
            queue: new_shared_queue(),
            stop_signal,
        }
    }

    pub fn queue(&self) -> &SharedTaskQueue {
        &self.queue
    }

    pub fn submit(&self, task: Task) -> String {
        let mut q = self.queue.write();
        q.enqueue(task)
    }

    pub fn cancel_all(&self) {
        let mut q = self.queue.write();
        q.clear();
        let _ = self.stop_signal.send(());
    }

    pub async fn execute_next(&self) -> Option<TaskResult> {
        let task = {
            let mut q = self.queue.write();
            q.dequeue()
        };

        let queued = task?;
        tracing::info!("Executing task: {} ({})", queued.task.name(), queued.id);

        let result = execute_task(&queued.task).await;

        if let Some(ref r) = result {
            tracing::info!(
                "Task {} completed: success={}, {}",
                queued.id,
                r.success,
                r.message
            );
        }

        result
    }
}

impl Default for TaskExecutor {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn execute_task(task: &Task) -> Option<TaskResult> {
    let bot_client = BOT_CLIENT.read().as_ref().cloned()?;
    let azalea = bot_client.azalea_client.read().clone()?;

    match task {
        Task::MoveTo { x, y, z } => {
            pathfinding::open_nearby_doors(&azalea, 3).await;
            let pos = azalea::Vec3::new(*x as f64, *y as f64, *z as f64);
            azalea.start_goto_with_opts(
                azalea::pathfinder::goals::RadiusGoal { pos, radius: 1.0 },
                pathfinding::smart_pathfinder_opts(),
            );
            Some(TaskResult::success(format!(
                "Moving to ({}, {}, {})",
                x, y, z
            )))
        }
        Task::Follow { player } => {
            bot_client.follow_stop.store(false, std::sync::atomic::Ordering::Relaxed);
            bot_client.set_following(Some(player.clone()));
            crate::bot::follow::start_following(
                azalea.clone(),
                player.clone(),
                bot_client.follow_stop.clone(),
            );
            Some(TaskResult::success(format!("Now following {}", player)))
        }
        Task::Mine { block, count } => {
            Some(TaskResult::success(format!(
                "Mining {}x {} - queued",
                count, block
            )))
        }
        Task::Place { block, x, y, z } => {
            azalea.chat(&format!("/setblock {} {} {} {}", x, y, z, block));
            Some(TaskResult::success(format!(
                "Placing {} at ({}, {}, {})",
                block, x, y, z
            )))
        }
        Task::Build {
            blueprint,
            origin_x,
            origin_y,
            origin_z,
        } => {
            let blueprint_path = std::path::Path::new(blueprint);
            match crate::blueprint::BlueprintLoader::load_from_file(blueprint_path) {
                Ok(bp) => {
                    let origin = (*origin_x, *origin_y, *origin_z);
                    let mut build_executor = crate::builder::BuildExecutor::new(
                        azalea.clone(),
                        bp,
                        origin,
                    );

                    let materials = build_executor.check_materials();
                    if !materials.is_complete() {
                        let missing: Vec<String> = materials.missing.materials
                            .iter()
                            .map(|(k, v)| format!("{}: {}", k, v))
                            .collect();
                        return Some(TaskResult::failure(format!(
                            "Missing materials: {}",
                            missing.join(", ")
                        )));
                    }

                    match build_executor.execute().await {
                        Ok(placed) => Some(TaskResult::success(format!(
                            "Built {}: {} blocks placed",
                            blueprint, placed
                        ))),
                        Err(e) => Some(TaskResult::failure(format!(
                            "Build failed: {}",
                            e
                        ))),
                    }
                }
                Err(e) => Some(TaskResult::failure(format!(
                    "Failed to load blueprint: {}",
                    e
                ))),
            }
        }
        Task::Attack { target } => {
            Some(TaskResult::success(format!("Attacking {} - queued", target)))
        }
        Task::Guard { player } => {
            bot_client.set_guarding(true);
            bot_client.set_master(Some(player.clone()));
            azalea.chat(&format!("I will protect you, {}!", player));
            Some(TaskResult::success(format!("Now protecting {}", player)))
        }
        Task::Craft { item, count } => {
            azalea.chat(&format!("/craft {} {}", item, count));
            Some(TaskResult::success(format!("Crafting {}x {}", count, item)))
        }
        Task::Sleep => {
            azalea.chat("/sleep");
            Some(TaskResult::success("Going to sleep".to_string()))
        }
        Task::Eat => {
            Some(TaskResult::success("Eating - queued".to_string()))
        }
        Task::CollectItem { item, count } => {
            Some(TaskResult::success(format!(
                "Collecting {}x {} - queued",
                count, item
            )))
        }
        Task::OpenChest { x, y, z } => {
            let pos = azalea::BlockPos::new(*x, *y, *z);
            azalea.block_interact(pos);
            Some(TaskResult::success(format!(
                "Opening chest at ({}, {}, {})",
                x, y, z
            )))
        }
        Task::Reply { message } => {
            azalea.chat(message);
            None
        }
        Task::ExecuteCommand { command } => {
            azalea.chat(&format!("/{}", command));
            Some(TaskResult::success(format!("Executed /{}", command)))
        }
        Task::Stop => {
            azalea.stop_pathfinding();
            bot_client.follow_stop.store(true, std::sync::atomic::Ordering::Relaxed);
            bot_client.set_guarding(false);
            bot_client.set_following(None);
            Some(TaskResult::success("Stopped all actions".to_string()))
        }
    }
}
