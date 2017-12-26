use serenity::model::Message;
use serenity::prelude::Context;
use database;

pub fn on_message(ctx: &Context, msg: &Message) {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    let guild_id = match msg.guild_id() {
        Some(guild) => guild.0,
        None => return,
    };

    let _ = pool.update_user_activity_message(msg.author.id.0);
}
