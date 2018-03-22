use serenity::prelude::Context;
use serenity::model::id::UserId;
use serenity::model::id::GuildId;
use serenity::model::guild::Guild;
use serenity::model::guild::PartialGuild;
use serenity::model::guild::Member;
use serenity::model::event::PresenceUpdateEvent;
use serenity::prelude::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use utils::config::get_pool;
use database::ConnectionPool;

pub fn on_guild_create(ctx: &Context, guild: &Guild, _: bool) {
    let pool = get_pool(ctx);

    if let Err(e) = pool.update_cache_guild(guild) {
        warn_discord!("[PLUGIN:db_cache] Error while updating cache_guild: {}", e);
    }

    let member_ids = guild.members.keys().map(|x| x.0).collect();
    if let Err(e) = pool.update_cache_members(guild.id.0, member_ids) {
        warn_discord!("[PLUGIN:db_cache] Error while updating cache_member: {}", e);
    }

    let users = guild.members.values().map(|x| (*x.user.read()).clone()).collect();
    if let Err(e) = pool.update_cache_users(users) {
        warn_discord!("[PLUGIN:db_cache] Error while updating cache_user: {}", e);
    }
}

pub fn on_guild_member_addition(_ctx: &Context, pool: &ConnectionPool, guild: &GuildId, member: &mut Member) {
    let user = member.user.read();

    if let Err(e) = pool.update_cache_members(guild.0, vec![user.id.0]) {
        warn_discord!("[PLUGIN:db_cache] Error while updating cache_member: {}", e);
    }

    if let Err(e) = pool.update_cache_users(vec![(*user).clone()]) {
        warn_discord!("[PLUGIN:db_cache] Error while updating cache_user: {}", e);
    }
}

pub fn on_guild_members_chunk(ctx: &Context, guild_id: &GuildId, members: &HashMap<UserId, Member>) {
    let pool = get_pool(ctx);
    let member_ids = members.keys().map(|x| x.0).collect();

    if let Err(e) = pool.update_cache_members(guild_id.0, member_ids) {
        warn_discord!("[PLUGIN:db_cache] Error while updating cache_member: {}", e);
    }

    let users = members.values().map(|x| (*x.user.read()).clone()).collect();
    if let Err(e) = pool.update_cache_users(users) {
        warn_discord!("[PLUGIN:db_cache] Error while updating cache_user: {}", e);
    }
}

pub fn on_guild_update(ctx: &Context, guild: &Option<Arc<RwLock<Guild>>>, _partial_guild: &PartialGuild) {
    let pool = get_pool(ctx);
    
    if let Some(ref guild) = *guild {
        let guild = guild.read();
        if let Err(e) = pool.update_cache_guild(&guild) {
            warn_discord!("[PLUGIN:db_cache] Error while updating cache_guild: {}", e);
        }
    }
}

pub fn on_presence_update(ctx: &Context, presence_event: &PresenceUpdateEvent) {
    let pool = get_pool(ctx);

    if let Some(ref user) = presence_event.presence.user {
        let user = user.read();

        if let Err(e) = pool.update_cache_users(vec![(*user).clone()]) {
            warn_discord!("[PLUGIN:db_cache] Error while updating cache_user: {}", e);
        }
    }
}
