use serenity::prelude::Context;
use serenity::model::gateway::Ready;
use serenity::client::CACHE;

use std;
use std::{thread, time};
use std::sync::{Once, ONCE_INIT};

use database;

static INIT: Once = ONCE_INIT;

pub fn on_ready(ctx: &Context, _: &Ready) {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap().clone();
    
    let mut count = 0;
    INIT.call_once(|| {
        thread::spawn(move || loop {
            let thirty_sec = time::Duration::from_secs(30);

            if let Ok(events) = pool.get_events() {
                if let Some(counter) = events.iter().find(|x| x.name == "PRESENCE_UPDATE") {
                    // kill self if presence_updates count haven't changed in past 30 seconds
                    if count == counter.count {
                        warn_discord!("PRESENCE_UPDATE has not changed in the past 30 seconds, exiting.");
                        std::process::exit(1);
                    }

                    debug!("presence updates: previous: {}, current: {}", count, counter.count);

                    count = counter.count;
                }
            }

            // Update stats for bot, probably should rename this file or something
            {
                let cache = CACHE.read();
                let guilds_count = cache.guilds.len();
                let channels_count = cache.channels.len();
                let users_count = cache.users.len();

                pool.update_stat("bot", "guilds_count", None, Some(guilds_count as i64));
                pool.update_stat("bot", "channels_count", None, Some(channels_count as i64));
                pool.update_stat("bot", "users_count", None, Some(users_count as i64));

                debug!("updated cached bot info: guilds: {}, channels: {}, users: {}",
                    guilds_count, channels_count, users_count);
            }

            thread::sleep(thirty_sec);
        });
    });
}
