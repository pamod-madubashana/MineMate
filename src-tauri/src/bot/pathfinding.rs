use azalea::block::BlockStates;
use azalea::pathfinder::PathfinderOpts;
use azalea::registry::builtin::BlockKind;
use azalea::Client;

use crate::config::AppConfig;

pub fn smart_pathfinder_opts() -> PathfinderOpts {
    let config = AppConfig::load().unwrap_or_default();
    PathfinderOpts::new()
        .allow_mining(config.pathfinding.allow_mining)
}

pub fn is_openable_block(kind: BlockKind) -> bool {
    matches!(
        kind,
        BlockKind::OakDoor
            | BlockKind::BirchDoor
            | BlockKind::SpruceDoor
            | BlockKind::JungleDoor
            | BlockKind::AcaciaDoor
            | BlockKind::DarkOakDoor
            | BlockKind::MangroveDoor
            | BlockKind::CherryDoor
            | BlockKind::BambooDoor
            | BlockKind::CrimsonDoor
            | BlockKind::WarpedDoor
            | BlockKind::IronDoor
            | BlockKind::OakFenceGate
            | BlockKind::BirchFenceGate
            | BlockKind::SpruceFenceGate
            | BlockKind::JungleFenceGate
            | BlockKind::AcaciaFenceGate
            | BlockKind::DarkOakFenceGate
            | BlockKind::MangroveFenceGate
            | BlockKind::CherryFenceGate
            | BlockKind::BambooFenceGate
            | BlockKind::CrimsonFenceGate
            | BlockKind::WarpedFenceGate
    )
}

pub async fn open_nearby_doors(bot: &Client, radius: i32) {
    let config = match AppConfig::load() {
        Ok(c) => c,
        Err(_) => return,
    };
    if !config.pathfinding.smart_doors {
        return;
    }

    let bot_pos = match bot.position() {
        Ok(p) => p,
        Err(_) => return,
    };

    let world = match bot.world() {
        Ok(w) => w,
        Err(_) => return,
    };

    let door_kinds = [
        BlockKind::OakDoor,
        BlockKind::BirchDoor,
        BlockKind::SpruceDoor,
        BlockKind::JungleDoor,
        BlockKind::AcaciaDoor,
        BlockKind::DarkOakDoor,
        BlockKind::MangroveDoor,
        BlockKind::CherryDoor,
        BlockKind::BambooDoor,
        BlockKind::CrimsonDoor,
        BlockKind::WarpedDoor,
        BlockKind::IronDoor,
        BlockKind::OakFenceGate,
        BlockKind::BirchFenceGate,
        BlockKind::SpruceFenceGate,
        BlockKind::JungleFenceGate,
        BlockKind::AcaciaFenceGate,
        BlockKind::DarkOakFenceGate,
        BlockKind::MangroveFenceGate,
        BlockKind::CherryFenceGate,
        BlockKind::BambooFenceGate,
        BlockKind::CrimsonFenceGate,
        BlockKind::WarpedFenceGate,
    ];

    let door_states = BlockStates::from(door_kinds);

    let origin = azalea::BlockPos::new(
        bot_pos.x as i32,
        bot_pos.y as i32,
        bot_pos.z as i32,
    );

    let mut doors_found = Vec::new();
    {
        let w = world.read();
        for dx in -radius..=radius {
            for dy in -2..=3 {
                for dz in -radius..=radius {
                    let pos = azalea::BlockPos::new(
                        origin.x + dx,
                        origin.y + dy,
                        origin.z + dz,
                    );
                    if let Some(state) = w.get_block_state(pos) {
                        if door_states.contains(&state) {
                            doors_found.push(pos);
                        }
                    }
                }
            }
        }
    }

    for door_pos in doors_found {
        bot.block_interact(door_pos);
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
}
