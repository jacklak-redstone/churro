use crate::proto::chatto::api::v1::*;
use crate::{Bot, CResult, ChurroError};

use std::sync::Arc;

pub struct Ctx {
    pub bot: Arc<Bot>,
    pub room: Option<Room>,
    pub user: Option<User>,
    pub message: Option<Message>,
    pub root_message: Option<Message>,
}

impl Ctx {
    pub fn with_bot(bot: Arc<Bot>) -> Self {
        Self {
            bot,
            room: None,
            user: None,
            message: None,
            root_message: None,
        }
    }

    pub async fn reply(&self, msg: &str, send_to_channel: bool) -> CResult {
        let Some(message) = &self.message else {
            return Err(ChurroError::ResourceNotFound("message"));
        };
        self.bot.reply(message, msg, send_to_channel).await?;
        Ok(())
    }

    pub async fn reply_in_thread(&self, msg: &str, send_to_channel: bool) -> CResult {
        let Some(message) = &self.message else {
            return Err(ChurroError::ResourceNotFound("message"));
        };
        self.bot
            .reply_in_thread(message, msg, send_to_channel)
            .await?;
        Ok(())
    }

    pub async fn send(&self, msg: &str, send_to_channel: bool) -> CResult {
        let Some(room) = &self.room else {
            return Err(ChurroError::ResourceNotFound("room"));
        };

        if let Some(message) = &self.message {
            if message.thread_root_event_id.is_empty() {
                self.bot.send_message(room, msg).await?;
            } else {
                self.bot
                    .send_in_thread(message, msg, send_to_channel)
                    .await?;
            }
        } else {
            self.bot.send_message(room, msg).await?;
        }

        Ok(())
    }

    pub async fn send_in_thread(&self, msg: &str, send_to_channel: bool) -> CResult {
        let Some(message) = &self.message else {
            return Err(ChurroError::ResourceNotFound("message"));
        };
        self.bot
            .send_in_thread(message, msg, send_to_channel)
            .await?;
        Ok(())
    }
}
