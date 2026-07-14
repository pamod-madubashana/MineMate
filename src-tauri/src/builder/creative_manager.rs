use azalea::Client;
use crate::bot::events::BotEvent;

const GIVE_WAIT_TICKS: usize = 5;
const GIVE_WAIT_MS: u64 = 300;

pub struct CreativeInventoryManager {
    bot: Client,
    current_item: Option<String>,
}

impl CreativeInventoryManager {
    pub fn new(bot: Client) -> Self {
        Self { bot, current_item: None }
    }

    pub async fn inject_item(&mut self, block_id: &str) -> Result<(), String> {
        let normalized = block_id.trim().to_lowercase();
        let with_prefix = if normalized.starts_with("minecraft:") {
            normalized.clone()
        } else {
            format!("minecraft:{}", normalized)
        };

        self.bot.chat(&format!("/give @s {} 1", with_prefix));
        self.bot.wait_updates(GIVE_WAIT_TICKS).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(GIVE_WAIT_MS)).await;

        self.bot.chat("/item replace entity @s weapon.mainhand with " .to_string() + &with_prefix);
        self.bot.wait_updates(GIVE_WAIT_TICKS).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(GIVE_WAIT_MS)).await;

        if let Some(bot_client) = crate::bot::handler::BOT_CLIENT.read().as_ref() {
            bot_client.emit_event(BotEvent::InventoryChanged);
        }

        self.current_item = Some(normalized);
        tracing::debug!("Equipped {} via /give", block_id);
        Ok(())
    }

    pub fn current_item(&self) -> Option<&str> {
        self.current_item.as_deref()
    }

    pub fn reset_item_cache(&mut self) {
        self.current_item = None;
    }
}
