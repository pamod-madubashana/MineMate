use azalea::Client;
use azalea::inventory::ItemStack;
use azalea::registry::builtin::ItemKind;
use azalea_protocol::packets::game::s_set_creative_mode_slot::ServerboundSetCreativeModeSlot;
use crate::bot::events::BotEvent;

const HOTBAR_SLOT: u16 = 0;
const INVENTORY_WAIT_TICKS: usize = 5;
const INVENTORY_WAIT_MS: u64 = 200;

pub struct CreativeInventoryManager {
    bot: Client,
}

impl CreativeInventoryManager {
    pub fn new(bot: Client) -> Self {
        Self { bot }
    }

    pub async fn inject_item(&self, item_kind: ItemKind, count: i32) -> Result<(), String> {
        let item_stack = ItemStack::new(item_kind, count);

        let packet = ServerboundSetCreativeModeSlot {
            slot_num: HOTBAR_SLOT,
            item_stack,
        };

        self.bot.write_packet(&packet);
        self.bot.wait_updates(INVENTORY_WAIT_TICKS).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(INVENTORY_WAIT_MS)).await;

        if let Some(bot_client) = crate::bot::handler::BOT_CLIENT.read().as_ref() {
            bot_client.emit_event(BotEvent::InventoryChanged);
        }

        tracing::debug!(
            "Injected {}x {} into hotbar slot {}",
            count,
            format!("{:?}", item_kind),
            HOTBAR_SLOT
        );
        Ok(())
    }

    pub async fn inject_grouped(
        &self,
        placements: &[(azalea::BlockPos, ItemKind)],
    ) -> Result<u32, String> {
        if placements.is_empty() {
            return Ok(0);
        }

        let mut injected = 0u32;
        let mut current_item = placements[0].1;
        let mut current_count = 1i32;

        for &(_, item_kind) in placements.iter().skip(1) {
            if item_kind == current_item {
                current_count += 1;
            } else {
                self.inject_item(current_item, current_count).await?;
                injected += current_count as u32;
                current_item = item_kind;
                current_count = 1;
            }
        }

        self.inject_item(current_item, current_count).await?;
        injected += current_count as u32;

        tracing::info!("Injected {} items in {} groups", injected, "batched");
        Ok(injected)
    }

    pub fn select_hotbar_slot(&self) {
        self.bot.set_selected_hotbar_slot(0);
    }

    pub fn current_slot_item(&self) -> Option<ItemKind> {
        self.bot.get_held_item().ok().and_then(|item| {
            if item.is_present() {
                Some(item.kind())
            } else {
                None
            }
        })
    }
}
