use serenity::prelude::Context;
use database;
use models::GuildConfig;

pub fn get_config_from_context(ctx: &Context, guild_id: u64) -> GuildConfig {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    pool.get_guild_config(guild_id)
}
