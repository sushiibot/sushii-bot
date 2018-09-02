use serenity::prelude::Context;
use serenity::model::gateway::Ready;

use std;
use std::{thread, time};
use std::sync::{Once, ONCE_INIT};
use parking_lot::deadlock;

use database;

static INIT: Once = ONCE_INIT;

pub fn on_ready(ctx: &Context, _: &Ready) {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap().clone();
    
    let mut count = 0;
    INIT.call_once(|| {
        thread::spawn(move || loop {
            let thirty_sec = time::Duration::from_secs(30);
            thread::sleep(thirty_sec);

            // Check for deadlocks
            let deadlocks = deadlock::check_deadlock();
            if !deadlocks.is_empty() {
                warn_discord!("{} deadlocks detected", deadlocks.len());
                for (i, threads) in deadlocks.iter().enumerate() {
                    println!("Deadlock #{}", i);
                    for t in threads {
                        println!("Thread Id {:#?}", t.thread_id());
                        println!("{:#?}", t.backtrace());
                    }
                }
            }


            // check if presences updated
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
        });
    });
}
