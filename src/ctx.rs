use crate::{Bot, CResult, ChurroError};
use crate::proto::chatto::api::v1::*;

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

    pub async fn reply(&self, msg: &str) -> CResult {
        let Some(message) = &self.message else { return Err(ChurroError::ResourceNotFound("message")) };
        self.bot.reply(message, msg).await?;
        Ok(())
    }
}
