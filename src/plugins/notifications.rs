use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::prelude::*;
use serenity::CACHE;

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

    let guild_id = match msg.guild_id {
        Some(guild) => guild.0,
        None => return,
    };

    if let Some(notifications) = pool.get_notifications(&msg.content.to_lowercase(), guild_id) {
        for notification in notifications {
            // skip notifications for self
            if notification.user_id as u64 == msg.author.id.0 {
                continue;
            }

            let channel = match CACHE
                .read()
                .guild_channel(msg.channel_id) {
                    Some(c) => c,
                    None => return, // return ok here, in dms
                };
            
            let channel_name = channel.read().name.clone();

            // check if can read channel
            if !channel
                .read()
                .permissions_for(notification.user_id as u64)
                .map(|permissions| permissions
                    .read_messages()
                )
                .unwrap_or(false) {
                    continue;
                }
            
            let guild = match CACHE
                .read()
                .guild(guild_id) {
                    Some(g) => g,
                    None => return, // return ok here too, in dms
                };
            
            let guild_name = guild.read().name.clone();

            // check if in guild
            if !guild
                .read()
                .members
                .contains_key(&UserId(notification.user_id as u64)) {
                    continue;
                }


            let lowered = msg.content.to_lowercase();

            if is_mid_word(&lowered, &notification.keyword) {
                continue; // should be continue not return to not skip other notifications
            }

            // message user
            let user = UserId(notification.user_id as u64);

            if let Ok(channel) = user.create_dm_channel() {
                let desc = format!(":speech_left: {} mentioned `{}` in #{} ({}) in {}",
                   msg.author.tag(), notification.keyword, channel_name, msg.channel_id.mention(), guild_name);

                // maybe switch to use Channel::messages() instead?
                // let mut messages = pool.get_messages(msg.channel_id.0, 3);
                let mut messages = CACHE
                    .read()
                    .messages
                    .get(&msg.channel_id)
                    .map(|channel| {
                        let mut v = channel
                            .values()
                            .cloned()
                            .collect::<Vec<Message>>();
                        
                        v.sort_by(|a, b| b.id.0.cmp(&a.id.0));
                        v.truncate(3);

                        v
                    });


                let sent_msg = channel.id.send_message(|m| m
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

                                let embed_or_image = if message.content.is_empty() {
                                    if !message.attachments.is_empty() {
                                        "[image attached]"
                                    } else if !message.embeds.is_empty() {
                                        "[embed attached]"
                                    } else {
                                        ""
                                    }
                                } else {
                                    ""
                                };


                                e = e.field(format!("[{}] {}", message.timestamp.format("%H:%M:%S UTC"), message.author.tag()),
                                    format!("> {}{}", content, embed_or_image),
                                    false);
                            }
                        } else {
                            let content = msg.content.replace(&notification.keyword, &format!("**{}**", notification.keyword));

                            // don't think embed_or_image needs to be used here since
                            // only message shown is the triggered message which should have content
                            e = e.field(format!("[{}] {}", msg.timestamp.format("%H:%M:%S UTC"), msg.author.tag()),
                                format!("> {}", content),
                                false);
                        }

                        e
                    })
                );

                if let Ok(mut sent_msg) = sent_msg {
                    let _ = sent_msg.edit(|m| m
                        .content(&format!(
                            "http://discordapp.com/channels/{}/{}/{}\n(Jump to message)", // guild, channel, message
                            guild_id,
                            msg.channel_id.0,
                            msg.id.0,
                        ))
                    );
                }

                pool.update_stat("notifications", "notifications_triggered", Some(1), None);
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