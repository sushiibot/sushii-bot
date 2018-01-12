use serenity::model::Message;
use serenity::prelude::Context;

use database::ConnectionPool;

pub fn on_message(_ctx: &Context, pool: &ConnectionPool, msg: &Message) {
    pool.log_message(msg);
}
