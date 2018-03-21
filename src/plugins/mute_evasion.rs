use serenity::model::guild::Member;
use serenity::model::id::{GuildId, RoleId};
use serenity::model::user::User;
use serenity::prelude::*;

use database::ConnectionPool;

pub fn on_guild_member_addition(_ctx: &Context, pool: &ConnectionPool, guild_id: &GuildId, member: &mut Member) {
    let user = member.user.read().clone();

    if pool.should_mute(user.id.0, guild_id.0) {
        let config = check_res!(pool.get_guild_config(guild_id.0));

        if let Some(role) = config.mute_role {
            let case_id = check_res!(pool.add_mod_action(
                "mute",
                guild_id.0,
                &user,
                Some("Automated Mute: User left with a mute role."),
                true,
                None,
            )).case_id;

            if let Err(e) = member.add_role(role as u64) {
                warn!("[plugin:mute_evasion] Error while adding mute role: {}", e);
                // remove failed mod entry
                pool.remove_mod_action(guild_id.0, &user, case_id);
            }
        }

        pool.delete_mute(user.id.0, guild_id.0)
    }
}

pub fn on_guild_member_removal(
    _ctx: &Context,
    pool: &ConnectionPool,
    guild_id: &GuildId,
    user: &User,
    member: &Option<Member>,
) {
    if let Some(ref memb) = *member {
        let config = check_res!(pool.get_guild_config(guild_id.0));

        // check if mute role set in config
        if let Some(role) = config.mute_role {
            // check if member left has the role
            if memb.roles.contains(&RoleId(role as u64)) {
                pool.add_mute(user.id.0, guild_id.0);
            }
        }
    }
}
