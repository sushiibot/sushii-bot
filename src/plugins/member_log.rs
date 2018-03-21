use serenity::model::guild::Member;
use serenity::model::id::{GuildId, ChannelId};
use serenity::model::user::User;
use serenity::prelude::*;

use database::ConnectionPool;
use utils::time::now_utc;

pub fn on_guild_member_addition(_ctx: &Context, pool: &ConnectionPool, guild_id: &GuildId, member: &Member) {
    let config = check_res!(pool.get_guild_config(guild_id.0));

    if let Some(log_channel) = config.log_member {
        let channel = ChannelId(log_channel as u64);

        let user = member.user.read().clone();

        let _ = channel.send_message(|m| m
            .embed(|e| e
                .author(|a|
                    a.name(&format!("{} ({})", user.tag(), user.id.0))
                    .icon_url(&user.face())
                )
                .color(0x2ecc71)
                .footer(|f| f
                    .text("User Joined")
                )
                .timestamp(now_utc().format("%Y-%m-%dT%H:%M:%S").to_string())
            )
        );

        pool.log_member_event(guild_id.0, user.id.0, "join");
    }
}

pub fn on_guild_member_removal(_ctx: &Context, pool: &ConnectionPool, guild_id: &GuildId, user: &User, _: &Option<Member>) {
    let config = check_res!(pool.get_guild_config(guild_id.0));

    if let Some(log_channel) = config.log_member {
        let channel = ChannelId(log_channel as u64);

        let _ = channel.send_message(|m| m
            .embed(|e| e
                .author(|a|
                    a.name(&format!("{} ({})", user.tag(), user.id.0))
                    .icon_url(&user.face())
                )
                .color(0xe67e22)
                .footer(|f| f
                    .text("User Left")
                )
                .timestamp(now_utc().format("%Y-%m-%dT%H:%M:%S").to_string())
            )
        );

        pool.log_member_event(guild_id.0, user.id.0, "leave");
    }
}
