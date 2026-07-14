#![allow(dead_code)]

use serde_json::{json, Value};

pub fn available_tools() -> Vec<Value> {
    vec![
        json!({
            "type": "function",
            "function": {
                "name": "move_to",
                "description": "Move the bot to specific coordinates",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "x": { "type": "integer", "description": "X coordinate" },
                        "y": { "type": "integer", "description": "Y coordinate" },
                        "z": { "type": "integer", "description": "Z coordinate" }
                    },
                    "required": ["x", "y", "z"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "follow",
                "description": "Follow a player",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "player": { "type": "string", "description": "Player name to follow" }
                    },
                    "required": ["player"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "mine",
                "description": "Mine blocks of a specific type",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "block": { "type": "string", "description": "Block type to mine" },
                        "count": { "type": "integer", "description": "Number of blocks to mine" }
                    },
                    "required": ["block", "count"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "craft",
                "description": "Craft an item",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "item": { "type": "string", "description": "Item to craft" },
                        "count": { "type": "integer", "description": "Number to craft" }
                    },
                    "required": ["item", "count"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "attack",
                "description": "Attack the nearest hostile entity",
                "parameters": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "place_block",
                "description": "Place a block at coordinates",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "block": { "type": "string", "description": "Block type to place" },
                        "x": { "type": "integer", "description": "X coordinate" },
                        "y": { "type": "integer", "description": "Y coordinate" },
                        "z": { "type": "integer", "description": "Z coordinate" }
                    },
                    "required": ["block", "x", "y", "z"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "build_structure",
                "description": "Build a structure from a blueprint at specified position. Origin defaults to current position if not specified.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "structure": { "type": "string", "description": "Blueprint name or file path to build" },
                        "x": { "type": "integer", "description": "X coordinate for build origin (optional, defaults to current position)" },
                        "y": { "type": "integer", "description": "Y coordinate for build origin (optional, defaults to current position)" },
                        "z": { "type": "integer", "description": "Z coordinate for build origin (optional, defaults to current position)" }
                    },
                    "required": ["structure"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "reply",
                "description": "Reply to a player in chat",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "message": { "type": "string", "description": "Message to send" }
                    },
                    "required": ["message"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "execute_command",
                "description": "Execute a server command (operator only)",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": { "type": "string", "description": "Command to execute" }
                    },
                    "required": ["command"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "scan_area",
                "description": "Scan nearby blocks in a radius",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "radius": { "type": "integer", "description": "Scan radius in blocks" }
                    },
                    "required": ["radius"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "give_item",
                "description": "Give an item to a player",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "player": { "type": "string", "description": "Player name" },
                        "item": { "type": "string", "description": "Item to give" },
                        "count": { "type": "integer", "description": "Number of items" }
                    },
                    "required": ["player", "item", "count"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "teleport",
                "description": "Teleport to coordinates",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "x": { "type": "integer", "description": "X coordinate" },
                        "y": { "type": "integer", "description": "Y coordinate" },
                        "z": { "type": "integer", "description": "Z coordinate" }
                    },
                    "required": ["x", "y", "z"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "sort_chests",
                "description": "Organize storage chests",
                "parameters": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "protect_player",
                "description": "Guard and protect a player",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "player": { "type": "string", "description": "Player to protect" }
                    },
                    "required": ["player"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "load_blueprint",
                "description": "Load a blueprint from file",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Path to blueprint file" }
                    },
                    "required": ["path"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "check_materials",
                "description": "Check if we have materials for a blueprint",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "blueprint": { "type": "string", "description": "Blueprint name or path" }
                    },
                    "required": ["blueprint"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "import_url",
                "description": "Import a blueprint from a URL (e.g., GrabCraft)",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "url": { "type": "string", "description": "URL to import from" },
                        "name": { "type": "string", "description": "Name for the blueprint" }
                    },
                    "required": ["url", "name"]
                }
            }
        }),
    ]
}
