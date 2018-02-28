use serenity::prelude::Context;
use serenity::model::id::UserId;
use serenity::model::id::GuildId;
use serenity::model::guild::Guild;
// use serenity::model::guild::PartialGuild;
use serenity::model::guild::Member;
// use serenity::prelude::RwLock;
use std::collections::HashMap;
// use std::sync::Arc;

use utils::config::get_pool;
use database::ConnectionPool;

pub fn on_guild_create(ctx: &Context, guild: &Guild, _: bool) {
    let pool = get_pool(&ctx);

    if let Err(e) = pool.update_cache_guild(guild) {
        warn_discord!("[PLUGIN:db_cache] Error while updating cache_guild: {}", e);
    }

    let member_ids = guild.members.keys().map(|x| x.0).collect();
    if let Err(e) = pool.update_cache_members(guild.id.0, member_ids) {
        warn_discord!("[PLUGIN:db_cache] Error while updating cache_member: {}", e);
    }
}

pub fn on_guild_member_addition(_ctx: &Context, pool: &ConnectionPool, guild: &GuildId, member: &mut Member) {
    let user_id = {
        member.user.read().id.0
    };

    if let Err(e) = pool.update_cache_members(guild.0, vec![user_id]) {
        warn_discord!("[PLUGIN:db_cache] Error while updating cache_member: {}", e);
    }
}

pub fn on_guild_members_chunk(ctx: &Context, guild_id: &GuildId, members: &HashMap<UserId, Member>) {
    let pool = get_pool(&ctx);
    let member_ids = members.keys().map(|x| x.0).collect();

    if let Err(e) = pool.update_cache_members(guild_id.0, member_ids) {
        warn_discord!("[PLUGIN:db_cache] Error while updating cache_member: {}", e);
    }
}

// fn on_guild_update(ctx: Context, _: Option<Arc<RwLock<Guild>>>, partial_guild: PartialGuild) {
//     let pool = get_pool(&ctx);
// 
//     if let Err(e) = pool.update_cache_guild(&guild) {
//         warn_discord!("[PLUGIN:db_cache] Error while updating cache_guild: {}", e);
//     }
// }