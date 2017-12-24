use serenity::framework::standard::CommandError;

use std::fmt::Write;
use database;

command!(add_notification(ctx, msg, args) {
    let keyword = match args.single::<String>() {
        Ok(val) => val,
        Err(_) => return Err(CommandError("Missing keyword".to_owned())),
    };

    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    let guild_id = match msg.guild_id() {
        Some(val) => val.0,
        None => return Err(CommandError("No guild".to_owned())),
    };

    pool.new_notification(msg.author.id.0, guild_id, &keyword);

    let s = format!("Added a new notification with keyword `{}`", keyword);
    let _ = msg.channel_id.say(&s);
});
