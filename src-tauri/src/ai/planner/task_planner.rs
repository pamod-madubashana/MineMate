use crate::ai::client::ToolCall;
use crate::task_engine::types::Task;

pub fn tool_call_to_task(tool_call: &ToolCall) -> Result<Task, String> {
    let args: serde_json::Value = serde_json::from_str(&tool_call.arguments)
        .map_err(|e| format!("Failed to parse arguments: {}", e))?;

    match tool_call.name.as_str() {
        "move_to" => {
            let x = args["x"].as_i64().ok_or("Missing x")? as i32;
            let y = args["y"].as_i64().ok_or("Missing y")? as i32;
            let z = args["z"].as_i64().ok_or("Missing z")? as i32;
            Ok(Task::MoveTo { x, y, z })
        }
        "follow" => {
            let player = args["player"].as_str().ok_or("Missing player")?.to_string();
            Ok(Task::Follow { player })
        }
        "mine" => {
            let block = args["block"].as_str().ok_or("Missing block")?.to_string();
            let count = args["count"].as_u64().unwrap_or(1) as u32;
            Ok(Task::Mine { block, count })
        }
        "place_block" => {
            let block = args["block"].as_str().ok_or("Missing block")?.to_string();
            let x = args["x"].as_i64().ok_or("Missing x")? as i32;
            let y = args["y"].as_i64().ok_or("Missing y")? as i32;
            let z = args["z"].as_i64().ok_or("Missing z")? as i32;
            Ok(Task::Place { block, x, y, z })
        }
        "build_structure" => {
            let structure = args["structure"].as_str().ok_or("Missing structure")?.to_string();
            Ok(Task::Build {
                blueprint: structure,
                origin_x: 0,
                origin_y: 0,
                origin_z: 0,
            })
        }
        "attack" => Ok(Task::Attack {
            target: "nearest".to_string(),
        }),
        "protect_player" => {
            let player = args["player"].as_str().ok_or("Missing player")?.to_string();
            Ok(Task::Guard { player })
        }
        "craft" => {
            let item = args["item"].as_str().ok_or("Missing item")?.to_string();
            let count = args["count"].as_u64().unwrap_or(1) as u32;
            Ok(Task::Craft { item, count })
        }
        "reply" => {
            let message = args["message"].as_str().ok_or("Missing message")?.to_string();
            Ok(Task::Reply { message })
        }
        "execute_command" => {
            let command = args["command"].as_str().ok_or("Missing command")?.to_string();
            Ok(Task::ExecuteCommand { command })
        }
        "teleport" => {
            let x = args["x"].as_i64().ok_or("Missing x")? as i32;
            let y = args["y"].as_i64().ok_or("Missing y")? as i32;
            let z = args["z"].as_i64().ok_or("Missing z")? as i32;
            Ok(Task::ExecuteCommand {
                command: format!("tp {} {} {}", x, y, z),
            })
        }
        "give_item" => {
            let player = args["player"].as_str().ok_or("Missing player")?.to_string();
            let item = args["item"].as_str().ok_or("Missing item")?.to_string();
            let count = args["count"].as_u64().unwrap_or(1) as u32;
            Ok(Task::ExecuteCommand {
                command: format!("give {} {} {}", player, item, count),
            })
        }
        _ => Err(format!("Unknown tool: {}", tool_call.name)),
    }
}

pub fn validate_task(task: &Task) -> Result<(), String> {
    match task {
        Task::MoveTo { x, y, z } => {
            if *y < -64 || *y > 320 {
                return Err(format!("Y coordinate {} is out of range (-64 to 320)", y));
            }
            Ok(())
        }
        Task::Mine { count, .. } => {
            if *count == 0 || *count > 64 {
                return Err(format!("Invalid count: {} (must be 1-64)", count));
            }
            Ok(())
        }
        Task::Place { y, .. } => {
            if *y < -64 || *y > 320 {
                return Err(format!("Y coordinate {} is out of range (-64 to 320)", y));
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
