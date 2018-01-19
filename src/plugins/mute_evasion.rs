use serenity::model::guild::Member;
use serenity::model::id::{GuildId, RoleId};
use serenity::model::user::User;
use serenity::prelude::*;

use utils::config::get_pool;

pub fn on_guild_member_addition(ctx: &Context, guild_id: &GuildId, member: &mut Member) {
    let pool = get_pool(&ctx);
    let user_id = member.user.read().id.0;

    if pool.should_mute(user_id, guild_id.0) {
        let config = pool.get_guild_config(guild_id.0);

        if let Some(role) = config.mute_role {
            if let Err(e) = member.add_role(role as u64) {
                warn!("[plugin:mute_evasion] Error while adding mute role: {}", e);
            }
        }

        pool.delete_mute(user_id, guild_id.0)
    }
}

pub fn on_guild_member_removal(
    ctx: &Context,
    guild_id: &GuildId,
    user: &User,
    member: &Option<Member>,
) {
    if let &Some(ref memb) = member {
        let pool = get_pool(&ctx);
        let config = pool.get_guild_config(guild_id.0);

        // check if mute role set in config
        if let Some(role) = config.mute_role {
            // check if member left has the role
            if memb.roles.contains(&RoleId(role as u64)) {
                pool.add_mute(user.id.0, guild_id.0);
            }
        }
    }
}
