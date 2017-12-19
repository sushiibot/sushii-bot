use serenity::model::GuildId;
use serenity::model::ChannelId;
use serenity::model::Member;
use serenity::model::User;
use serenity::model::ReactionType;
use serenity::prelude::*;

use util::get_config_from_context;

pub fn on_guild_member_addition(ctx: &Context, guild_id: &GuildId, member: &Member) {
    let config = get_config_from_context(&ctx, guild_id.0);

    if let Some(joinmsg) = config.join_msg.clone() {
        if let Some(msgchannel) = config.msg_channel.clone() {
            let channel = match ChannelId(msgchannel as u64).get() {
                Ok(val) => val.id(),
                Err(_) => return,
            };

            let _ = channel.send_message(|m| {
                let mut m = m.content(joinmsg);

                if let Some(join_react) = config.join_react.clone() {
                    m = m.reactions(vec![ReactionType::from(join_react)])
                }

                m
            });
        }
    }
}

pub fn on_guild_member_removal(ctx: &Context, guild_id: &GuildId, user: &User, _: &Option<Member>) {
    let config = get_config_from_context(&ctx, guild_id.0);

    if let Some(leavemsg) = config.leave_msg.clone() {
        if let Some(msgchannel) = config.msg_channel.clone() {
            let channel = match ChannelId(msgchannel as u64).get() {
                Ok(val) => val.id(),
                Err(_) => return,
            };

            let _ = channel.send_message(|m| m.content(leavemsg));
        }
    }
}
