use serenity::model::channel::Message;
use serenity::prelude::Context;
use database::ConnectionPool;

pub fn on_message(_ctx: &Context, pool: &ConnectionPool, msg: &Message) {
    let _ = pool.update_user_activity_message(msg.author.id.0);
}
