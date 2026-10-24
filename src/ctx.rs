use crate::Bot;
use crate::proto::chatto::api::v1::*;
use crate::{CResult, ChurroError};

use std::sync::Arc;

pub struct Ctx {
    pub bot: Arc<Bot>,
    pub room: Option<Room>,
    pub user: Option<User>,
    pub message: Option<Message>,
    pub root_message: Option<Message>,
}

impl Ctx {
    /// Create a new context containing only the given bot.
    pub fn with_bot(bot: Arc<Bot>) -> Self {
        Self {
            bot,
            room: None,
            user: None,
            message: None,
            root_message: None,
        }
    }

    /// Send a reply to the current message (if any).
    pub async fn reply(&self, msg: &str, send_to_channel: bool) -> CResult {
        let Some(message) = &self.message else {
            return Err(ChurroError::ResourceNotFound("message"));
        };
        self.bot.client.reply(message, msg, send_to_channel).await?;
        Ok(())
    }

    /// Send a reply in the same thread as the current message.
    pub async fn reply_in_thread(&self, msg: &str, send_to_channel: bool) -> CResult {
        let Some(message) = &self.message else {
            return Err(ChurroError::ResourceNotFound("message"));
        };
        self.bot
            .client
            .reply_in_thread(message, msg, send_to_channel)
            .await?;
        Ok(())
    }

    /// Send a message to the current room, handling thread context.
    pub async fn send(&self, msg: &str, send_to_channel: bool) -> CResult {
        let Some(room) = &self.room else {
            return Err(ChurroError::ResourceNotFound("room"));
        };

        if let Some(message) = &self.message {
            if message.thread_root_event_id.is_empty() {
                self.bot.client.send_message(room, msg).await?;
            } else {
                self.bot
                    .client
                    .send_in_thread(message, msg, send_to_channel)
                    .await?;
            }
        } else {
            self.bot.client.send_message(room, msg).await?;
        }

        Ok(())
    }

    /// Send a message as a reply in the current thread.
    pub async fn send_in_thread(&self, msg: &str, send_to_channel: bool) -> CResult {
        let Some(message) = &self.message else {
            return Err(ChurroError::ResourceNotFound("message"));
        };
        self.bot
            .client
            .send_in_thread(message, msg, send_to_channel)
            .await?;
        Ok(())
    }

    pub async fn react(&self, emoji: &str) -> CResult {
        let Some(message) = &self.message else {
            return Err(ChurroError::ResourceNotFound("message"));
        };
        self.bot.client.react(message, emoji).await?;
        Ok(())
    }
}
