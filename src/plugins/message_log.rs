use serenity::model::channel::Message;
use serenity::model::event::MessageUpdateEvent;
use serenity::prelude::Context;

use database::ConnectionPool;

pub fn on_message(_ctx: &Context, pool: &ConnectionPool, msg: &Message) {
    pool.log_message(msg);
}

pub fn on_message_update(_ctx: &Context, pool: &ConnectionPool, msg_update: &MessageUpdateEvent) {
    // update database when message is edited
    if let Some(ref content) = msg_update.content {
        pool.update_message(msg_update.id.0, content);
    }
}
