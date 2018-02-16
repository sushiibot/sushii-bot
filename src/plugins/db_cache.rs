use serenity::prelude::Context;
use serenity::model::guild::Guild;
use serenity::model::guild::PartialGuild;
use serenity::prelude::RwLock;
use std::sync::Arc;

use utils::config::get_pool;

pub fn on_guild_create(ctx: &Context, guild: &Guild, _: bool) {
    let pool = get_pool(&ctx);

    if let Err(e) = pool.update_cache_guild(guild) {
        warn_discord!("[PLUGIN:db_cache] Error while updating cache_guild: {}", e);
    }
}

// fn on_guild_update(ctx: Context, _: Option<Arc<RwLock<Guild>>>, partial_guild: PartialGuild) {
//     let pool = get_pool(&ctx);
// 
//     if let Err(e) = pool.update_cache_guild(&guild) {
//         warn_discord!("[PLUGIN:db_cache] Error while updating cache_guild: {}", e);
//     }
// }