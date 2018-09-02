use serenity::model::channel::Message;
use serenity::model::id::ChannelId;
use serenity::model::id::MessageId;
use serenity::model::id::UserId;
use serenity::prelude::Context;

use database::ConnectionPool;
use utils::config::get_config;
use utils::time::now_utc;

pub fn on_message(_ctx: &Context, pool: &ConnectionPool, msg: &Message) {
    pool.log_message(msg);
}

pub fn on_message_update(ctx: &Context, pool: &ConnectionPool, msg_update: &Message) {
    if msg_update.author.bot {
        return;
    }

    // get server config

    let msg = match pool.get_message(msg_update.id.0) {
        Some(m) => m,
        None => return,
    };

    if msg.bot {
        return;
    }

    let guild_id = match msg.guild {
        Some(g) => g,
        None => return,
    };

    // ignore some lazy load or embed change
    if msg_update.content == msg.content {
        return;
    }

    let config = check_res!(get_config(ctx, pool, guild_id as u64));

    let (tag, face) = if let Ok(user) = UserId(msg.author as u64).to_user() {
        (user.tag(), user.face())
    } else {
        ("N/A".into(), "https://cdn.discordapp.com/embed/avatars/1.png".into())
    };

    if let Some(channel) = config.log_msg {
        let now = now_utc();

        let _ = ChannelId(channel as u64).send_message(|m| m
            .embed(|e| e
                .title("Message Edited")
                .author(|a| a
                    .name(&format!("{} ({})", tag, msg.author))
                    .icon_url(&face)
                )
                .colour(0x9b59b6)
                .field("Old", &msg.content, false)
                .field("New", &msg_update.content, false)
                .field("Channel", &format!("<#{}>", msg.channel), false)
                .footer(|f| f
                    .text("Edited at")
                )
                .timestamp(now.format("%Y-%m-%dT%H:%M:%S").to_string())
            )
        );
    }

    // update database when message is edited
    pool.update_message(msg_update.id.0, &msg_update.content);
}

pub fn on_message_delete(ctx: &Context, pool: &ConnectionPool, _channel_id: &ChannelId, msg_id: &MessageId) {
    let msg = match pool.get_message(msg_id.0) {
        Some(m) => m,
        None => return,
    };

    if msg.bot {
        return;
    }

    let guild_id = match msg.guild {
        Some(id) => id,
        None => return,
    };

    // get server config
    let config = check_res!(get_config(ctx, pool, guild_id as u64));

    if let Some(channel) = config.log_msg {
        let (tag, face) = if let Ok(user) = UserId(msg.author as u64).to_user() {
            (user.tag(), user.face())
        } else {
            ("N/A".into(), "https://cdn.discordapp.com/embed/avatars/1.png".into())
        };

        let now = now_utc();

        let _ = ChannelId(channel as u64).send_message(|m| m
            .embed(|e| e
                .title("Message Deleted")
                .author(|a| a
                    .name(&format!("{} ({})", tag, msg.author))
                    .icon_url(&face)
                )
                .colour(0xe74c3c)
                .field("Message Content", msg.content, false)
                .field("Channel", &format!("<#{}>", msg.channel), false)
                .footer(|f| f
                    .text("Deleted at")
                )
                .timestamp(now.format("%Y-%m-%dT%H:%M:%S").to_string())
            )
        );
    }
}