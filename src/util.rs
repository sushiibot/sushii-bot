use chrono::{DateTime, Utc};
use chrono::naive::NaiveDateTime;
use serenity::prelude::Context;
use database;
use models::GuildConfig;

pub fn get_id(value: &str) -> Option<u64> {
    // check if it's already an ID
    if let Ok(id) = value.parse::<u64>() {
        return Some(id);
    }

    // Derived from https://docs.rs/serenity/0.4.5/src/serenity/utils/mod.rs.html#158-172
    if value.starts_with("<@!") {
        let len = value.len() - 1;
        value[3..len].parse::<u64>().ok()
    } else if value.starts_with("<@") {
        let len = value.len() - 1;
        value[2..len].parse::<u64>().ok()
    } else {
        None
    }
}

pub fn get_now_utc() -> NaiveDateTime {
    // get current timestamp
    Utc::now().naive_utc()
}

pub fn get_config_from_context(ctx: &Context, guild_id: u64) -> GuildConfig {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap();

    pool.get_guild_config(guild_id)
}
