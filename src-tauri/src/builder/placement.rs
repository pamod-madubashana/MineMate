use azalea::Client;
use crate::blueprint::types::BlockPlacement;

pub async fn place_block(bot: &Client, placement: &BlockPlacement) -> Result<(), String> {
    bot.chat(&format!(
        "/setblock {} {} {} {}",
        placement.x, placement.y, placement.z, placement.block_id
    ));
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    Ok(())
}

pub async fn place_blocks(bot: &Client, placements: &[BlockPlacement]) -> Result<u32, String> {
    let mut placed = 0;
    for placement in placements {
        match place_block(bot, placement).await {
            Ok(()) => { placed += 1; }
            Err(e) => {
                tracing::warn!("Failed to place block at ({}, {}, {}): {}", placement.x, placement.y, placement.z, e);
            }
        }
    }
    Ok(placed)
}
