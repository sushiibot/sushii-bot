use serenity::model::channel::Message;
use serenity::model::event::MessageUpdateEvent;
use serenity::model::id::ChannelId;
use serenity::model::id::MessageId;
use serenity::prelude::Context;

use database::ConnectionPool;
use utils::config::get_config;

pub fn on_message(_ctx: &Context, pool: &ConnectionPool, msg: &Message) {
    pool.log_message(msg);
}

pub fn on_message_update(ctx: &Context, pool: &ConnectionPool, msg_update: &MessageUpdateEvent) {
    if let Some(ref user) = msg_update.author {
        if user.bot {
            return;
        }
    }

    if let Some(ref content) = msg_update.content {
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

        let updated_content = match msg_update.content {
            Some(ref c) => c,
            None => return,
        };

        // ignore some lazy load or embed change
        if *updated_content == msg.content {
            return;
        }

        let config = check_res!(get_config(ctx, pool, guild_id as u64));

        if let Some(channel) = config.log_msg {
            let s = format!("`[{}] {} ({})` edited message in <#{}>:\n{}\n->\n{}",
                msg.created.format("%Y-%m-%d %H:%M:%S UTC"),
                msg.tag, msg.author, msg.channel,
                msg.content, updated_content
            );
            let _ = ChannelId(channel as u64).say(s);
        }

        // update database when message is edited
        pool.update_message(msg_update.id.0, content);
    }
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
        let s = format!("`[{}] {} ({})` deleted message in <#{}>:\n{}",
            msg.created.format("%Y-%m-%d %H:%M:%S UTC"),
            msg.tag, msg.author, msg.channel,
            msg.content
        );
        let _ = ChannelId(channel as u64).say(s);
    }
}