use serenity::framework::standard::CommandError;

use std::fmt::Write;
use database;

command!(rank(ctx, msg) {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    let guild_id = match msg.guild_id() {
        Some(guild) => guild.0,
        None => return Err(CommandError("No guild found.".to_owned())),
    };

    let level_data = match pool.get_level(msg.author.id.0, guild_id) {
        Some(level_data) => level_data,
        None => return Err(CommandError("No level data found.".to_owned())),
    };

    let mut s = "```ruby\nMessage Count\n".to_owned();
    let _ = write!(s, "Month: {}\n", level_data.msg_month);
    let _ = write!(s, "Week: {}\n", level_data.msg_week);
    let _ = write!(s, "Day: {}\n", level_data.msg_day);
    let _ = write!(s, "All Time: {}\n", level_data.msg_all_time);
    let _ = write!(s, "```");

    let _ = msg.channel_id.say(&s);
});
