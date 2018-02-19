use serenity::model::channel::Message;
use serenity::prelude::Context;
use database::ConnectionPool;

pub fn on_message(_ctx: &Context, pool: &ConnectionPool, msg: &Message) {
    let guild_id = match msg.guild_id() {
        Some(guild) => guild.0,
        None => return,
    };

    // ignore bots
    if msg.author.bot {
        return;
    }

    if let Err(e) = pool.update_level(msg.author.id.0, guild_id) {
        let e = format!("[PLUGIN:levels] Error updating user level: {}", e);
        warn_discord!(&e);
    }

    pool.update_stat("xp", "given");
}
