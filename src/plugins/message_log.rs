use serenity::model::channel::Message;
use serenity::model::event::MessageUpdateEvent;
use serenity::model::id::ChannelId;
use serenity::model::id::MessageId;
use serenity::model::id::UserId;
use serenity::prelude::Context;

use serde_json;

use database::ConnectionPool;
use utils::config::get_config;
use utils::time::now_utc;

pub fn on_message(_ctx: &Context, pool: &ConnectionPool, msg: &Message) {
    pool.log_message(msg);
}

pub fn on_message_update(ctx: &Context, pool: &ConnectionPool, event: &MessageUpdateEvent) {
    // get server config

    let msg = match pool.get_message(event.id.0) {
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

    let content = match event.content {
        Some(ref c) => c.clone(),
        None => return,
    };

    // ignore some lazy load or embed change
    if content == msg.content {
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
                .field("New", &content, false)
                .field("Channel", &format!("<#{}>", msg.channel), false)
                .footer(|f| f
                    .text("Edited at")
                )
                .timestamp(now.format("%Y-%m-%dT%H:%M:%S").to_string())
            )
        );
    }

    // update database when message is edited
    pool.update_message(event.id.0, &content);
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

    let discord_msg = if let Some(msg) = msg.msg.clone() {
        serde_json::from_value(msg).ok()
    } else {
        None
    };

    // get server config
    let config = check_res!(get_config(ctx, pool, guild_id as u64));

    let attachments = discord_msg
        .map(|msg: Message| msg.attachments
            .iter()
            .map(|attachment| attachment.url.clone())
            .collect::<Vec<String>>()
            .join("\n")
        )
        .and_then(|attachments| if attachments.is_empty() { None } else { Some(attachments) });

    let content = if msg.content.is_empty() { "N/A".into() } else { msg.content.clone() };

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
                .field("Message Content", content, false)
                .field("Attachments", attachments.unwrap_or("N/A".into()), false)
                .field("Channel", &format!("<#{}>", msg.channel), false)
                .footer(|f| f
                    .text("Deleted at")
                )
                .timestamp(now.format("%Y-%m-%dT%H:%M:%S").to_string())
            )
        );
    }
}