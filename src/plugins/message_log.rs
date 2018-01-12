use serenity::model::Message;
use serenity::prelude::Context;

use utils::config::get_pool;

pub fn on_message(ctx: &Context, msg: &Message) {
    let pool = get_pool(&ctx);

    pool.log_message(msg);
}
