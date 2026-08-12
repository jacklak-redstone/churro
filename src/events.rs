use crate::Ctx;
use async_trait::async_trait;

#[async_trait]
pub trait EventHandler: Send + Sync + 'static {
    async fn message_sent(&self, _: Ctx) {}
    async fn message_edited(&self, _: Ctx) {}
    async fn message_deleted(&self, _: Ctx, _: &str) {}

    async fn bot_mentioned(&self, _: Ctx) {}
    async fn dm_started(&self, _: Ctx) {}

    async fn reaction_added(&self, _: Ctx, _: &str) {}
    async fn reaction_removed(&self, _: Ctx, _: &str) {}
}
