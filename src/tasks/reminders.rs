use std::{thread, time};

use serenity::prelude::Context;
use serenity::model::gateway::Ready;
use serenity::http;

use chrono::{DateTime, Utc};
use timeago;
use std::sync::{Once, ONCE_INIT};

use database;

static INIT: Once = ONCE_INIT;

pub fn on_ready(ctx: &Context, _: &Ready) {
    let mut data = ctx.data.lock();
    let pool = data.get_mut::<database::ConnectionPool>().unwrap().clone();
    INIT.call_once(|| {
        thread::spawn(move || loop {
            let ten_sec = time::Duration::from_secs(10);
            thread::sleep(ten_sec);

            if let Some(reminders) = pool.get_overdue_reminders() {
                // loop through reminders
                for remind in reminders {
                    // get user by id
                    if let Ok(user) = http::get_user(remind.user_id as u64) {
                        let mut f = timeago::Formatter::new();
                        f.num_items(3);
                        f.ago("");

                        let ht = f.convert_chrono(
                            DateTime::<Utc>::from_utc(remind.time_set, Utc),
                            DateTime::<Utc>::from_utc(remind.time_to_remind, Utc),
                        );

                        let s =
                            format!(
                            "Ding dong! The reminder you set {} has expired \n```{}```",
                            ht,
                            remind.description,
                        );
                        if let Err(why) = user.direct_message(|m| m.content(&s)) {
                            error!(
                                "Failed to send message to {} for reminder: {}\n{}",
                                user.tag(),
                                remind.description,
                                why
                            );
                        }
                    }

                    pool.remove_reminder(remind.id);
                }
            }
        });
    });
}
