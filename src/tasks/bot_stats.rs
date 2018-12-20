use serenity::prelude::Context;
use serenity::model::gateway::Ready;
use serenity::CACHE;

use std::{thread, time};
use std::sync::{Once, ONCE_INIT};

use database;

static INIT: Once = ONCE_INIT;

pub fn on_ready(ctx: &Context, _: &Ready) {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap().clone();

    INIT.call_once(|| {
        debug!("Spawning stats thread");
        thread::spawn(move || loop {
            // sleep before checking, allows some time for guild lazy loading
            let five_min = time::Duration::from_secs(300);
            thread::sleep(five_min);

            // Update stats for bot, probably should rename this file or something
            {
                let cache = CACHE.read();
                let guilds_count = cache.guilds.len();
                let channels_count = cache.channels.len();
                let users_count = cache.guilds
                    .values()
                    .fold(0, |acc, x| acc + x.read().member_count);

                pool.update_stat("bot", "guilds_count", None, Some(guilds_count as i64));
                pool.update_stat("bot", "channels_count", None, Some(channels_count as i64));
                pool.update_stat("bot", "users_count", None, Some(users_count as i64));

                debug!("updated cached bot info: guilds: {}, channels: {}, users: {}",
                    guilds_count, channels_count, users_count);
            }
        });
    });
}
