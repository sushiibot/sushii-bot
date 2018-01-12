use serenity::model::Message;
use serenity::prelude::Context;
use database::ConnectionPool;

pub fn on_message(_ctx: &Context, pool: &ConnectionPool, msg: &Message) {
    let guild_id = match msg.guild_id() {
        Some(guild) => guild.0,
        None => return,
    };

    let _ = pool.update_level(msg.author.id.0, guild_id);
}
