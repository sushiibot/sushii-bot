use serenity::model::channel::Message;
use serenity::prelude::Context;

use database::ConnectionPool;
use std::env;
use utils::config::get_config;

pub fn on_message(ctx: &Context, pool: &ConnectionPool, msg: &Message) {
    // ignore bots
    if msg.author.bot {
        return;
    }

    let guild_id = match msg.guild_id() {
        Some(val) => val.0,
        None => return,
    };

    // returns if no guild or error
    let config = check_res!(get_config(ctx, pool, guild_id));

    // check if disabled channel
    if let Some(disabled_channels) = config.disabled_channels {
        if disabled_channels.contains(&(msg.channel_id.0 as i64)) {
            return;
        }
    }

    // check if in roles channel
    if let Some(channel) = config.role_channel {
        if channel == msg.channel_id.0 as i64 {
            return;
        }
    }

    let prefix = config.prefix.unwrap_or_else(|| env::var("DEFAULT_PREFIX").expect("Expected DEFAULT_PREFIX in the environment."));

    // check if starts with prefix
    if !msg.content.starts_with(&prefix) {
        return;
    }

    let tag_start = prefix.len();

    // check if there is a tag
    let mut tag_name = &msg.content[tag_start..];

    // check if space between prefix and tag name
    tag_name = if tag_name.starts_with(' ') {
        &tag_name[1..]
    } else {
        &tag_name[..]
    };

    // return silently if not found
    let found_tag = match pool.get_tag(guild_id, tag_name) {
        Some(val) => val,
        None => return,
    };

    // clean content
    let content = found_tag.content
        .replace("@everyone", "@\u{200b}everyone") // add zws to everyone and here mentions
        .replace("@here", "@\u{200b}here");

    // print content with zws space in front to prevent bot triggers
    let _ = msg.channel_id.say(&format!("\u{200b}{}", content));
    // update the counter
    pool.increment_tag(guild_id, tag_name);
}
