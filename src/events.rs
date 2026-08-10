use crate::Ctx;
use async_trait::async_trait;

#[async_trait]
pub trait EventHandler: Send + Sync + 'static {
    async fn message_sent(&self, _: Ctx) {}
    async fn bot_mentioned(&self, _: Ctx) {}
}
