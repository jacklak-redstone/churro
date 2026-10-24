use crate::Ctx;
use async_trait::async_trait;

#[async_trait]
pub trait EventHandler: Send + Sync + 'static {
    /// Called when a message is sent.
    async fn message_sent(&self, _: Ctx) {}
    /// Called when a message is edited.
    async fn message_edited(&self, _: Ctx) {}
    /// Called when a message is deleted.
    async fn message_deleted(&self, _: Ctx, _: &str) {}

    /// Called when a reaction is added.
    async fn reaction_added(&self, _: Ctx, _: &str) {}
    /// Called when a reaction is removed.
    async fn reaction_removed(&self, _: Ctx, _: &str) {}

    /// Called when a message is pinned.
    async fn message_pinned(&self, _: Ctx) {}
    /// Called when a message is unpinned.
    async fn message_unpinned(&self, _: Ctx) {}
}
