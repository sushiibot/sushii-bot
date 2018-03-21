use serenity::model::guild::Member;
use serenity::model::id::{GuildId, ChannelId};
use serenity::model::user::User;
use serenity::model::channel::ReactionType;
use serenity::prelude::*;

use database::ConnectionPool;

pub fn on_guild_member_addition(_ctx: &Context, pool: &ConnectionPool, guild_id: &GuildId, member: &Member) {
    let config = check_res!(pool.get_guild_config(guild_id.0));

    if let Some(joinmsg) = config.join_msg.clone() {
        if let Some(msgchannel) = config.msg_channel {
            let channel = ChannelId(msgchannel as u64);

            let user = member.user.read().clone();

            let _ = channel.send_message(|m| {
                let msg = format_message(&joinmsg, guild_id, &user);

                let mut m = m.content(msg);

                if let Some(join_react) = config.join_react.clone() {
                    m = m.reactions(vec![ReactionType::from(join_react)])
                }

                m
            });
        }
    }
}

pub fn on_guild_member_removal(_ctx: &Context, pool: &ConnectionPool, guild_id: &GuildId, user: &User, _: &Option<Member>) {
    let config = check_res!(pool.get_guild_config(guild_id.0));

    if let Some(leavemsg) = config.leave_msg.clone() {
        if let Some(msgchannel) = config.msg_channel {
            let channel = ChannelId(msgchannel as u64);

            let msg = format_message(&leavemsg, guild_id, user);

            let _ = channel.say(&msg);
        }
    }
}

/// Formats a string for join / leave messages, replaces placeholders for
/// member name, mention, and guild names
fn format_message(msg: &str, guild: &GuildId, user: &User) -> String {
    let guild_name = match guild.find() {
        Some(guild) => guild.read().name.to_owned(),
        None => {
            match guild.get() {
                Ok(guild) => guild.name,
                Err(_) => "".to_owned(),
            }
        }
    };


    let mut s = msg.replace("<mention>", &user.mention());
    s = s.replace("<username>", &user.name);
    s = s.replace("<server>", &guild_name);

    s
}
