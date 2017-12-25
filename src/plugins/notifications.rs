use serenity::model::Message;
use serenity::model::UserId;
use serenity::prelude::*;

use database;

pub fn on_message(ctx: &Context, msg: &Message) {
    // skip empty messages, images / embeds / etc
    if msg.content.is_empty() {
        return;
    }

    if msg.author.bot {
        return;
    }

    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    let guild_id = match msg.guild_id() {
        Some(guild) => guild.0,
        None => return,
    };

    if let Some(notifications) = pool.get_notifications(&msg.content, guild_id) {
        for notification in notifications {
            // skip notifications for self
            if notification.user_id as u64 == msg.author.id.0 {
                continue;
            }

            // message user
            let user = UserId(notification.user_id as u64);

            if let Ok(channel) = user.create_dm_channel() {
                let s = format!(
                    "{0} mentioned `{1}` in {2}:\n```{0}: {3}```",
                    msg.author.tag(),
                    notification.keyword,
                    msg.channel_id.mention(),
                    msg.content
                );
                let _ = channel.id.say(&s);
            } else {
                let s = format!(
                    "Failed sending notification message to: {}",
                    &notification.user_id.to_string()
                );
                error!("{}", s);
            }
        }
    }
}
