use serenity::prelude::Context;
use database;
use GuildConfigCache;
use diesel::result::Error;
use models::GuildConfig;
use reqwest;
use std::sync::Arc;
use keys::Reqwest;

pub fn get_config_from_context(ctx: &Context, guild_id: u64) -> Result<GuildConfig, Error> {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    pool.get_guild_config(guild_id)
}

pub fn get_pool(ctx: &Context) -> database::ConnectionPool {
    let mut data = ctx.data.lock();
    data.get_mut::<database::ConnectionPool>().unwrap().clone()
}


pub fn get_reqwest_client(ctx: &Context) -> Arc<reqwest::Client> {
    let data = ctx.data.lock();
    data.get::<Reqwest>().unwrap().clone()
}

pub fn get_config(ctx: &Context, pool: &database::ConnectionPool, guild_id: u64) -> Result<GuildConfig, Error> {
    let mut data = ctx.data.lock();
    let config_cache = data.get_mut::<GuildConfigCache>().unwrap();

    // search cache first
    if let Some(config) = config_cache.get(&guild_id) {
        return Ok(config.clone());
    }

    // not found in cache, get config from db
    let config = pool.get_guild_config(guild_id)?;
    debug!("Fetched config from database for guild: {}", guild_id);
    
    // update the cache with config from db
    config_cache.insert(guild_id, config.clone());
    debug!("Added config cache for guild: {}", guild_id);

    Ok(config.clone())
}

pub fn update_config(ctx: &Context, pool: &database::ConnectionPool, config: &GuildConfig) {
    // save pool to db
    pool.save_guild_config(config);

    // update config cache
    let mut data = ctx.data.lock();
    let config_cache = data.get_mut::<GuildConfigCache>().unwrap();

    // insert the new config into cache, this either adds new or updates old
    config_cache.insert(config.id as u64, config.clone());

    debug!("Updated config cache for guild: {}", config.id);
}
