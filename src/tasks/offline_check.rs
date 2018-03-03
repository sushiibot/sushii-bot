use serenity::prelude::Context;
use serenity::model::gateway::Ready;

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
            let five_min = time::Duration::from_secs(300);

            if let Ok(events) = pool.get_events() {
                if let Some(counter) = events.iter().find(|ref x| x.name == "PRESENCE_UPDATE") {
                    // kill self if presence_updates count haven't changed
                    // in the past 5 minutes
                    if count == counter.count {
                        warn_discord!("PRESENCE_UPDATE has not changed in the past 5 minutes, exiting.");
                        std::process::exit(1);
                    }

                    count = counter.count;
                }
            }

            thread::sleep(five_min);
        });
    });
}
