use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::prelude::*;

use database::ConnectionPool;
use utils::time::now_utc;

pub fn on_message(_ctx: &Context, pool: &ConnectionPool, msg: &Message) {
    // skip empty messages, images / embeds / etc
    if msg.content.is_empty() {
        return;
    }

    if msg.author.bot {
        return;
    }

    let guild_id = match msg.guild_id() {
        Some(guild) => guild.0,
        None => return,
    };

    if let Some(notifications) = pool.get_notifications(&msg.content.to_lowercase(), guild_id) {
        for notification in notifications {
            // skip notifications for self
            if notification.user_id as u64 == msg.author.id.0 {
                continue;
            }

            let lowered = msg.content.to_lowercase();

            if is_mid_word(&lowered, &notification.keyword) {
                return;
            }

            // message user
            let user = UserId(notification.user_id as u64);

            if let Ok(channel) = user.create_dm_channel() {
                let desc = format!(":speech_left: Your notification `{}` was triggered in {}", notification.keyword, msg.channel_id.mention());

                // maybe switch to use Channel::messages() instead?
                let mut messages = pool.get_messages(msg.channel_id.0, 3);


                let _ = channel.id.send_message(|m| m
                    .embed(|e| {
                        let mut e = e.color(0xf58b28)
                        .description(desc)
                        .timestamp(now_utc().format("%Y-%m-%dT%H:%M:%S").to_string());

                        if let Some(ref mut messages) = messages {
                            messages.reverse();
                            for message in messages {
                                let lowered = message.content.to_lowercase();
                                if !is_mid_word(&lowered, &notification.keyword) {

                                }
                                // bold the keyword
                                let content = if is_mid_word(&lowered, &notification.keyword) {
                                    message.content.clone()
                                } else if let Some(start) = lowered.rfind(&notification.keyword) {
                                    let end = start + notification.keyword.len();

                                    let mut content = message.content.clone();
                                    content.insert_str(end, "**");
                                    content.insert_str(start, "**");

                                    content
                                } else {
                                    message.content.replace(&notification.keyword, &format!("**{}**", notification.keyword))
                                };


                                e = e.field(format!("[{}] {}", message.created.format("%H:%M:%S UTC"), message.tag),
                                    format!("> {}", content),
                                    false);
                            }
                        } else {
                            let content = msg.content.replace(&notification.keyword, &format!("**{}**", notification.keyword));

                            e = e.field(format!("[{}] {}", msg.timestamp.format("%H:%M:%S UTC"), msg.author.tag()),
                                format!("> {}", content),
                                false);
                        }

                        e
                    })
                );
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


fn is_mid_word(msg: &str, search: &str) -> bool {
    let start = match msg.rfind(&search) {
        Some(val) => val,
        None => return true, // should be able to find, this shouldn't ever happen
    };
    let end = start + search.len();

    if start > 0 {
        let before = match msg.chars().nth(start - 1) {
            Some(val) => val,
            None => return true,
        };
        if before.is_alphanumeric() {
            return true;
        }
    }

    if end < msg.len() {
        let after = match msg.chars().nth(end) {
            Some(val) => val,
            None => return true,
        };
        if after.is_alphanumeric() {
            return true;
        }
    }

    false
}