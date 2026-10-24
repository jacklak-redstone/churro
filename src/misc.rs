use crate::proto::chatto::api::v1::*;

pub trait UserUtil {
    fn is_bot(&self) -> bool;
}

impl UserUtil for User {
    fn is_bot(&self) -> bool {
        self.bot.is_set()
    }
}
